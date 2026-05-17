use extism_pdk::*;

/// Forwards the JSON dispatch envelope to the Java host (JAR handlers via host function).
#[host_fn]
extern "ExtismHost" {
    fn portaki_gateway_dispatch(input: String) -> String;
}

#[plugin_fn]
pub fn portaki_query(input: String) -> FnResult<String> {
    Ok(unsafe { portaki_gateway_dispatch(input)? })
}

#[plugin_fn]
pub fn portaki_command(input: String) -> FnResult<String> {
    Ok(unsafe { portaki_gateway_dispatch(input)? })
}
