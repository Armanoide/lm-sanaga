use crate::token::token_generated_info::TokenGeneratedInfo;

/*pub fn tokens_stats_average(
    list: &Vec<TokenGeneratedInfo>,
    skip_first: bool,
) -> Option<(f64, f64, f64, f64)> {
    let data = if skip_first && list.len() > 1 {
        &list[1..]
    } else {
        list.as_slice()
    };

    if data.is_empty() {
        return None;
    }

    let len = data.len() as f64;

    let (total_gen_tps, total_prompt_tps, total_peak_mem, generation_tokens): (f64, f64, f64, f64) =
        data.iter()
            .map(|info| {
                (
                    info.generation_tps,
                    info.prompt_tps,
                    info.peak_memory as f64,
                    info.generation_tokens as f64,
                )
            })
            .fold((0.0, 0.0, 0.0, 0.0), |acc, x| {
                (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2, acc.3 + x.3)
            });

    Some((
        /*avg_generation_tps:*/ total_gen_tps / len,
        /*avg_prompt_tps:*/ total_prompt_tps / len,
        /* avg_peak_memory:*/ total_peak_mem / len,
        /* avg_generation_tokens: */ generation_tokens,
    ))
}*/

/*pub fn tokens_stats_average_formatted(
    list: &Vec<TokenGeneratedInfo>,
    skip_first: bool,
) -> Option<String> {
    if let (Some(stats), Some(first_response)) =
        (tokens_stats_average(list, skip_first), list.get(0))
    {
        let (avg_generation_tps, avg_prompt_tps, avg_peak_memory, total_generation_tokens) = stats;
        let output = format!(
            "\n====== Generation Summary\n\
             Average Prompt TPS     : {:>8.2} t/s\n\
             Average Generation TPS : {:>8.2} t/s\n\
             Average Peak Memory    : {:>8.2} MB\n\
             Total Generation Tokens: {:>8} tokens\n\
             ====== First Token\n\
             Prompt TPS             : {:>8.2} t/s\n\
             Tokens                 : {:>8} tokens\n\
             ======\n",
            avg_prompt_tps,
            avg_generation_tps,
            avg_peak_memory / (1024.0 * 1024.0),
            total_generation_tokens,
            first_response.prompt_tps,
            first_response.prompt_tokens,
        );
        Some(output)
    } else {
        None
    }
}*/
