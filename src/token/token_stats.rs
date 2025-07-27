use crate::token::token_generated_info::TokenGeneratedInfo;

pub fn tokens_stats_average(
    list: &Vec<TokenGeneratedInfo>,
    skip_first: bool,
) -> Option<(f64, f64, f64)> {
    let data = if skip_first && list.len() > 1 {
        &list[1..]
    } else {
        list.as_slice()
    };

    if data.is_empty() {
        return None;
    }

    let len = data.len() as f64;

    let (total_gen_tps, total_prompt_tps, total_peak_mem): (f64, f64, f64) = data
        .iter()
        .map(|info| {
            (
                info.generation_tps,
                info.prompt_tps,
                info.peak_memory as f64,
            )
        })
        .fold((0.0, 0.0, 0.0), |acc, x| {
            (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2)
        });

    Some((
        /*avg_generation_tps:*/ total_gen_tps / len,
        /*avg_prompt_tps:*/ total_prompt_tps / len,
        /* avg_peak_memory:*/ total_peak_mem / len,
    ))
}
