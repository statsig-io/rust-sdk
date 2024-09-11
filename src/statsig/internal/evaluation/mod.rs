pub use eval_result::EvalResult;
pub use eval_details::EvalDetails;
pub use eval_details::EvaluationReason;
pub use statsig_evaluator::StatsigEvaluator;

pub mod eval_details;
pub mod specs;

mod client_init_response_formatter;
mod country_lookup;
mod eval_helpers;
mod eval_result;
mod statsig_evaluator;
mod statsig_user_eval_ext;
mod ua_parser;
