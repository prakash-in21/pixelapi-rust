use pixelapi::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("PIXELAPI_KEY")
        .expect("set PIXELAPI_KEY env var (get one free at https://pixelapi.dev/app)");
    let client = Client::new(api_key);

    let r = client.generate("product photo of red sneakers, white background, studio lighting")?;
    println!("generated: {} (used {} credits)", r.output_url, r.credits_used);
    client.save(&r, "out/sneakers.png")?;

    let r = client.remove_background("https://pixelapi.dev/demo/photo.jpg")?;
    println!("bg-removed: {} (used {} credits)", r.output_url, r.credits_used);

    Ok(())
}
