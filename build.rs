
fn main() {
    println!(
        "cargo:rustc-env=SUPABASE_URL={}",
        std::env::var("SUPABASE_URL").unwrap_or_default()
    );

    println!(
        "cargo:rustc-env=SUPABASE_ANON_KEY={}",
        std::env::var("SUPABASE_ANON_KEY").unwrap_or_default()
    );
}
