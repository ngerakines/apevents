use actix_web::guard::{Guard, GuardContext};
use reqwest::header;

#[allow(non_snake_case)]
pub fn HeaderStart(name: &'static str, value: &'static str) -> impl Guard {
    HeaderStartGuard(
        header::HeaderName::try_from(name).unwrap(),
        value.to_string(),
        // header::HeaderValue::from_static(value),
    )
}

struct HeaderStartGuard(header::HeaderName, String);

impl Guard for HeaderStartGuard {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        if let Some(val) = ctx.head().headers.get(&self.0) {
            let values: Vec<String> = val
                .to_str()
                .unwrap()
                .split(",")
                .map(|s| s.to_string())
                .collect();
            return values.contains(&self.1);
        }
        false
    }
}
