use std::collections::HashMap;

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::network::{Headers, SetExtraHttpHeadersParams};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::{Page, ScreenshotParams};

use crate::config::types::{Action, Size};
use crate::error::XsnapError;

/// Sets the viewport (device metrics) for the given page.
pub async fn set_viewport(page: &Page, size: &Size) -> Result<(), XsnapError> {
    let params = SetDeviceMetricsOverrideParams::new(
        size.width as i64,
        size.height as i64,
        1.0,   // device_scale_factor
        false, // mobile
    );

    page.execute(params)
        .await
        .map_err(|e| XsnapError::CdpError {
            message: format!("Failed to set viewport: {}", e),
        })?;

    Ok(())
}

/// Navigates the page to the given URL.
pub async fn navigate(page: &Page, url: &str) -> Result<(), XsnapError> {
    page.goto(url)
        .await
        .map_err(|e| XsnapError::NavigationFailed {
            url: url.to_string(),
            message: format!("{}", e),
        })?;

    Ok(())
}

/// Sets extra HTTP headers to be sent with every request from this page.
pub async fn set_extra_headers(
    page: &Page,
    headers: &HashMap<String, String>,
) -> Result<(), XsnapError> {
    if headers.is_empty() {
        return Ok(());
    }

    let header_map: serde_json::Map<String, serde_json::Value> = headers
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();

    let params =
        SetExtraHttpHeadersParams::new(Headers::new(serde_json::Value::Object(header_map)));

    page.execute(params)
        .await
        .map_err(|e| XsnapError::CdpError {
            message: format!("Failed to set HTTP headers: {}", e),
        })?;

    Ok(())
}

/// Captures a screenshot of the current page as PNG bytes.
///
/// If `full_page` is true, the entire scrollable page is captured.
pub async fn capture_screenshot(page: &Page, full_page: bool) -> Result<Vec<u8>, XsnapError> {
    let params = ScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png)
        .full_page(full_page)
        .build();

    let bytes = page
        .screenshot(params)
        .await
        .map_err(|e| XsnapError::ScreenshotFailed {
            message: format!("{}", e),
        })?;

    Ok(bytes)
}

/// Extracts the size restriction list from an action, if present.
fn size_restriction(action: &Action) -> Option<&Vec<String>> {
    match action {
        Action::Wait {
            size_restriction, ..
        } => size_restriction.as_ref(),
        Action::Click {
            size_restriction, ..
        } => size_restriction.as_ref(),
        Action::Type {
            size_restriction, ..
        } => size_restriction.as_ref(),
        Action::Scroll {
            size_restriction, ..
        } => size_restriction.as_ref(),
        Action::ForcePseudoState {
            size_restriction, ..
        } => size_restriction.as_ref(),
        Action::Function {
            size_restriction, ..
        } => size_restriction.as_ref(),
    }
}

/// Checks whether an action should be executed for the given viewport size.
///
/// If the action has a size restriction list, the current size name must be
/// in that list. If there is no restriction, the action always executes.
fn should_execute_for_size(action: &Action, current_size: &str) -> bool {
    match size_restriction(action) {
        Some(sizes) => sizes.iter().any(|s| s == current_size),
        None => true,
    }
}

/// Executes a single action on the page.
///
/// Actions with a size restriction that does not include `current_size` are
/// skipped. The `Function` action variant is a no-op here because function
/// expansion is handled by the runner before actions reach this point.
pub async fn execute_action(
    page: &Page,
    action: &Action,
    current_size: &str,
) -> Result<(), XsnapError> {
    if !should_execute_for_size(action, current_size) {
        return Ok(());
    }

    match action {
        Action::Wait { timeout, .. } => {
            tokio::time::sleep(std::time::Duration::from_millis(*timeout)).await;
        }

        Action::Click { selector, .. } => {
            let element = page
                .find_element(selector)
                .await
                .map_err(|e| XsnapError::CdpError {
                    message: format!("Failed to find element '{}': {}", selector, e),
                })?;

            element.click().await.map_err(|e| XsnapError::CdpError {
                message: format!("Failed to click element '{}': {}", selector, e),
            })?;
        }

        Action::Type { selector, text, .. } => {
            let element = page
                .find_element(selector)
                .await
                .map_err(|e| XsnapError::CdpError {
                    message: format!("Failed to find element '{}': {}", selector, e),
                })?;

            element.click().await.map_err(|e| XsnapError::CdpError {
                message: format!("Failed to focus element '{}': {}", selector, e),
            })?;

            element
                .type_str(text)
                .await
                .map_err(|e| XsnapError::CdpError {
                    message: format!("Failed to type into '{}': {}", selector, e),
                })?;
        }

        Action::Scroll {
            selector,
            px_amount,
            ..
        } => {
            if let Some(sel) = selector {
                // Scroll a specific element into view.
                let element = page
                    .find_element(sel)
                    .await
                    .map_err(|e| XsnapError::CdpError {
                        message: format!("Failed to find element '{}': {}", sel, e),
                    })?;

                element
                    .scroll_into_view()
                    .await
                    .map_err(|e| XsnapError::CdpError {
                        message: format!("Failed to scroll element '{}' into view: {}", sel, e),
                    })?;
            } else if let Some(amount) = px_amount {
                // Scroll the window by a pixel amount.
                let js = format!("window.scrollBy(0, {})", amount);
                page.evaluate(js).await.map_err(|e| XsnapError::CdpError {
                    message: format!("Failed to scroll by {} px: {}", amount, e),
                })?;
            }
        }

        Action::ForcePseudoState {
            selector,
            hover,
            active,
            focus,
            visited,
            ..
        } => {
            // Build the list of pseudo states to force.
            let mut states = Vec::new();
            if *hover {
                states.push("hover");
            }
            if *active {
                states.push("active");
            }
            if *focus {
                states.push("focus");
            }
            if *visited {
                states.push("visited");
            }

            // Use JavaScript to apply pseudo states via the CSS OM API.
            // This approach uses CDP's CSS.forcePseudoState indirectly through
            // DOM operations: first we need the element's node, then we force
            // the pseudo state via page.execute with ForcePseudoStateParams.
            //
            // However, a simpler approach is to use JS to find the element and
            // apply classes/attributes. For true CDP pseudo state forcing, we
            // need the DOM nodeId, which requires enable DOM domain and query.
            //
            // We use the CDP approach: find element, get its nodeId via
            // describe, then call CSS.forcePseudoState.
            let element = page
                .find_element(selector)
                .await
                .map_err(|e| XsnapError::CdpError {
                    message: format!(
                        "Failed to find element for pseudo state '{}': {}",
                        selector, e
                    ),
                })?;

            let node_id = element.node_id;

            let params = chromiumoxide::cdp::browser_protocol::css::ForcePseudoStateParams::new(
                node_id,
                states.iter().map(|s| s.to_string()).collect(),
            );

            page.execute(params)
                .await
                .map_err(|e| XsnapError::CdpError {
                    message: format!("Failed to force pseudo state on '{}': {}", selector, e),
                })?;
        }

        Action::Function { .. } => {
            // Function actions are expanded before execution by the runner.
            // This is a no-op at the browser action level.
        }
    }

    Ok(())
}
