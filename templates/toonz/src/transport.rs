use crate::{
	Params,
	helpers::{Content, check},
};
use aidoku::{
	Result,
	alloc::{String, format},
	helpers::uri::encode_uri_component,
	imports::{js::WebView, net::Request, std::sleep},
	prelude::*,
};

fn response(
	params: &Params,
	url: &str,
	content: Option<&Content>,
	rsc: bool,
	navigation_url: Option<&str>,
) -> Result<(i32, String)> {
	if !params.browser_requests {
		let mut request = match content {
			Some(content) => content.request(url)?,
			None => Request::get(url)?,
		};
		if rsc {
			request.set_header("RSC", "1");
		}
		let response = request.send()?;
		let status = response.status_code();
		if status == 404 && rsc {
			return Ok((status, String::new()));
		}
		return Ok((status, check(response)?.get_string()?));
	}

	let view = WebView::new();
	view.load_html("<html><body></body></html>", Some(params.base_url))?;
	let base = serde_json::to_string(params.base_url)?;
	let mut ready = false;
	for _ in 0..5 {
		if view.eval(&format!(
			"String(document.baseURI === {base} + '/' && document.readyState !== 'loading')"
		))? == "true"
		{
			ready = true;
			break;
		}
		sleep(1);
	}
	if !ready {
		bail!("Toonz could not start its browser request");
	}
	let result = browser_response(&view, url, content, rsc)?;
	if result.0 == 403
		&& let Some(navigation_url) = navigation_url
	{
		navigate(&view, navigation_url, rsc)?;
		return browser_response(&view, url, content, rsc);
	}
	Ok(result)
}

fn browser_response(
	view: &WebView,
	url: &str,
	content: Option<&Content>,
	rsc: bool,
) -> Result<(i32, String)> {
	let cookie = if let Some(content) = content {
		let prefs = serde_json::json!({
			"rating": content.rating,
			"types": content.types,
			"blockedGenres": content.blocked,
		});
		Some(format!(
			"reader_prefs={}; Path=/; Secure; SameSite=Lax",
			encode_uri_component(serde_json::to_string(&prefs)?)
		))
	} else {
		None
	};
	let config = serde_json::json!({ "url": url, "cookie": cookie, "rsc": rsc });
	let script = format!(
		"({})({})",
		r#"(config) => {
	if (config.cookie !== null) {
		document.cookie = config.cookie;
	}
	window.toonzResponse = null;
	const request = new XMLHttpRequest();
	request.open("GET", config.url, true);
	request.timeout = 15000;
	if (config.rsc) {
		request.setRequestHeader("RSC", "1");
	}
	request.onload = () => {
		window.toonzResponse = [request.status, request.responseText];
	};
	request.onerror = () => {
		window.toonzResponse = [0, "Network request failed"];
	};
	request.ontimeout = () => {
		window.toonzResponse = [0, "Request timed out"];
	};
	request.onabort = () => {
		window.toonzResponse = [0, "Request cancelled"];
	};
	window.toonzRequest = request;
	request.send();
	return "started";
}"#,
		serde_json::to_string(&config)?
	);
	view.eval(&script)?;
	for _ in 0..20 {
		let result = view.eval("JSON.stringify(window.toonzResponse)")?;
		if result != "null" {
			let (status, body): (i32, String) = serde_json::from_str(&result)?;
			if status == 0 {
				bail!("{body}: {url}");
			}
			return Ok((status, body));
		}
		sleep(1);
	}
	view.eval("window.toonzRequest.abort(); 'cancelled'")?;
	bail!("Toonz request timed out: {url}");
}

pub fn get(params: &Params, url: &str, content: Option<&Content>) -> Result<String> {
	let (status, body) = response(params, url, content, false, None)?;
	if !(200..300).contains(&status) {
		bail!("The site answered HTTP {status}: {url}");
	}
	Ok(body)
}

pub fn reader(params: &Params, url: &str) -> Result<Option<String>> {
	let (status, body) = response(params, url, None, true, Some(url))?;
	if status == 404 {
		return Ok(None);
	}
	if !(200..300).contains(&status) {
		bail!("The site answered HTTP {status}: {url}");
	}
	Ok(Some(body))
}

fn navigate(view: &WebView, url: &str, reader: bool) -> Result<(String, String)> {
	view.load(Request::get(url)?)?;
	let selector = if reader {
		"img[src*='/uploads/chapters/']"
	} else {
		"[data-comic-id]"
	};
	let script = format!(
		r#"(() => {{
			if (document.readyState === 'loading' || !document.querySelector({})) return '';
			return JSON.stringify([location.href, document.documentElement.outerHTML]);
		}})()"#,
		serde_json::to_string(selector)?
	);
	for _ in 0..25 {
		if let Ok(result) = view.eval(&script)
			&& !result.is_empty()
		{
			let page = serde_json::from_str(&result)?;
			view.eval("window.stop(); 'stopped'")?;
			return Ok(page);
		}
		sleep(1);
	}
	view.eval("window.stop(); 'stopped'")?;
	bail!("Toonz could not load this page in its browser: {url}");
}

pub fn series(params: &Params, url: &str) -> Result<(String, String)> {
	if !params.browser_requests {
		return Ok((String::from(url), get(params, url, None)?));
	}
	navigate(&WebView::new(), url, false)
}

pub fn chapters(params: &Params, url: &str, series_url: &str) -> Result<String> {
	let (status, body) = response(params, url, None, false, Some(series_url))?;
	if !(200..300).contains(&status) {
		bail!("The site answered HTTP {status}: {url}");
	}
	Ok(body)
}
