use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::{Deserialize, Serialize};

use crate::error::AuraResult;
use crate::indexer::get_all_items;

/// Rank range for items (1.0 = baseline, 5.0 = maximum boosted).
const MAX_RANK: f64 = 5.0;
/// Baseline rank assigned to newly indexed items.
const BASE_RANK: f64 = 1.0;
/// Scale denominator: maps rank range [BASE..MAX] to frequency bonus [0..1].
const RANK_RANGE: f64 = MAX_RANK - BASE_RANK;

/// A single search result returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub kind: String,
    pub score: f64,
    pub rank: f64,
}

/// Fuzzy search across the index.
/// Returns up to `max_results` items ordered by composite score.
pub fn fuzzy_search(query: &str, max_results: usize) -> AuraResult<Vec<SearchResult>> {
    let query = query.trim();
    if query.is_empty() {
        // Return top-ranked items when query is empty
        let items = get_all_items()?;
        let results = items
            .into_iter()
            .take(max_results)
            .map(|item| SearchResult {
                id: item.id,
                title: item.title.clone(),
                path: item.path.clone(),
                kind: item.kind.clone(),
                score: item.rank,
                rank: item.rank,
            })
            .collect();
        return Ok(results);
    }

    let matcher = SkimMatcherV2::default().smart_case();
    let items = get_all_items()?;

    let mut scored: Vec<(i64, SearchResult)> = items
        .into_iter()
        .filter_map(|item| {
            let match_score = matcher.fuzzy_match(&item.title, query).unwrap_or(0);
            if match_score <= 0 {
                return None;
            }

            // Composite: MatchQuality*0.7 + FrequencyBonus*0.2 + RecencyBonus*0.1
            let match_quality = match_score as f64 / 200.0;
            let frequency_bonus = (item.rank - BASE_RANK).max(0.0) / RANK_RANGE;
            let recency_bonus = item
                .last_modified
                .map(|ts| {
                    let age_days = (chrono::Utc::now().timestamp() - ts) as f64 / 86400.0;
                    (1.0 / (1.0 + age_days / 30.0)).min(1.0)
                })
                .unwrap_or(0.5);

            let composite =
                match_quality * 0.7 + frequency_bonus * 0.2 + recency_bonus * 0.1;

            Some((
                match_score,
                SearchResult {
                    id: item.id,
                    title: item.title,
                    path: item.path,
                    kind: item.kind,
                    score: composite,
                    rank: item.rank,
                },
            ))
        })
        .collect();

    // Sort by composite score descending, then by raw match score
    scored.sort_by(|a, b| {
        b.1.score
            .partial_cmp(&a.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.0.cmp(&a.0))
    });

    let results = scored
        .into_iter()
        .take(max_results)
        .map(|(_, r)| r)
        .collect();

    Ok(results)
}
