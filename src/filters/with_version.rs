use crate::fault::Fault;
use warp::{reject, Filter, Rejection};
const PROJECT_NAME: &str = "primecrime";

pub fn with_version() -> impl Filter<Extract = (u8,), Error = Rejection> + Clone {
    warp::header::optional::<String>("Accept") //application/vnd.heimstaden.v1+json
        .and_then(|o: Option<String>| async move {
            if let Some(s) = o {
                // Parse header.
                let start = format!("application/vnd.{}.v", PROJECT_NAME);
                if s == "*/*" {
                    // Default accept header.
                    Ok(0) // Use zero version.
                } else if s.starts_with(&start) && s.ends_with("+json") {
                    let g: String = s
                        .chars()
                        .skip(start.len())
                        .take(s.chars().count() - (start.len() + 5))
                        .collect(); // Magic number is the 5 character in +json
                    match g.parse::<u8>() {
                        Ok(v) => Ok(v),
                        Err(_) => Err(reject::custom(Fault::IllegalArgument(format!(
                            "Could not parse Accept header ({}).",
                            s
                        )))),
                    }
                } else {
                    Err(reject::custom(Fault::IllegalArgument(format!(
                        "Malformed Accept header ({}).",
                        s
                    ))))
                }
            } else {
                Ok(0) // Use zero version.
            }
        })
}
