use warp::{Filter, Rejection};
use tera::{Tera, Context};
use std::sync::Arc;
use std::collections::HashMap;
use serde::Deserialize;
use std::convert::Infallible;

#[derive(Debug)]
struct TeraError(String);

impl warp::reject::Reject for TeraError {}

#[tokio::main]
async fn main() {
    let tera = Arc::new(Tera::new("templates/**/*.html").unwrap());

    // Serve static files (e.g., CSS) from the "static" directory
    let static_files = warp::fs::dir("static");

    let index = warp::get()
        .and(warp::path::end())
        .and(with_tera(tera.clone()))
        .and_then(|tera: Arc<Tera>| {
            async move {
                let context = Context::new();
                let rendered = tera.render("index.html", &context)
                    .map_err(|e| warp::reject::custom(TeraError(format!("Failed to render 'index.html': {:?}", e))))?;
                Ok::<_, Rejection>(warp::reply::html(rendered))
            }
        });

    let search = warp::get()
        .and(warp::path("search"))
        .and(warp::query::<SearchQuery>())
        .and(with_tera(tera.clone()))
        .and_then(|query: SearchQuery, tera: Arc<Tera>| {
            async move {
                let mut context = Context::new();
                context.insert("query", &query.category);

                let quotes: HashMap<&str, Vec<&str>> = HashMap::from([
                    ("motivational", vec!["The best way to predict the future is to invent it.", "You miss 100% of the shots you don't take.", "Believe you can and you're halfway there.", "Success is not final, failure is not fatal: It is the courage to continue that counts.", "Your limitation—it's only your imagination."]),
                    ("love", vec!["Love all, trust a few, do wrong to none.", "To love and be loved is to feel the sun from both sides.", "The greatest thing you'll ever learn is just to love and be loved in return.", "Love is composed of a single soul inhabiting two bodies.", "I have waited for this opportunity for more than half a century, to repeat to you once again my vow of eternal fidelity and everlasting love."]),
                    ("humor", vec!["I told my wife she was drawing her eyebrows too high. She looked surprised.", "I threw a boomerang a year ago. I live in constant fear.", "Why don’t skeletons fight each other? They don’t have the guts.", "I’m on a whiskey diet. I’ve lost three days already.", "I used to play piano by ear, but now I use my hands."]),
                    ("wisdom", vec!["Wisdom is the reward you get for a lifetime of listening when you'd have preferred to talk.", "The only true wisdom is in knowing you know nothing.", "The fool doth think he is wise, but the wise man knows himself to be a fool.", "Wisdom begins in wonder.", "In three words I can sum up everything I've learned about life: it goes on."]),
                    ("friendship", vec!["A real friend is one who walks in when the rest of the world walks out.", "Friendship is born at that moment when one person says to another, 'What! You too? I thought I was the only one.'", "Friends are the siblings God never gave us.", "A friend is someone who knows all about you and still loves you.", "True friends are never apart, maybe in distance but never in heart."]),
                ]);

                let default_quotes = vec!["No quotes found"];
                let quotes_for_category = quotes.get(query.category.as_str()).unwrap_or(&default_quotes);

                context.insert("quotes", quotes_for_category);

                let rendered = tera.render("search.html", &context)
                    .map_err(|e| warp::reject::custom(TeraError(format!("Failed to render 'search.html': {:?}", e))))?;
                Ok::<_, Rejection>(warp::reply::html(rendered))
            }
        });

    let routes = index.or(search).or(static_files);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    category: String,
}

fn with_tera(tera: Arc<Tera>) -> impl Filter<Extract = (Arc<Tera>,), Error = Infallible> + Clone {
    warp::any().map(move || tera.clone())
}
