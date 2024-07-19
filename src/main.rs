use std::{env, fs::File, io::Read};
use reqwest::blocking::Client;
use serde_json::json;
use std::error::Error;
use base64::{engine::general_purpose, Engine};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Feature {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct Image {
    content: String,
}

#[derive(Serialize)]
struct Request {
    image: Image,
    features: Vec<Feature>,
}

#[derive(Serialize)]
struct VisionRequest {
    requests: Vec<Request>,
}

#[derive(Deserialize, Debug)]
struct EntityAnnotation {
    description: String,
    score: f64,
}

#[derive(Deserialize, Debug)]
struct AnnotateImageResponse {
    labelAnnotations: Option<Vec<EntityAnnotation>>,
}

#[derive(Deserialize, Debug)]
struct VisionResponse {
    responses: Vec<AnnotateImageResponse>,
}

fn make_vision_request(api_key: &str, image_base64: &str) -> Result<VisionResponse, Box<dyn Error>> {
    let url = format!("https://vision.googleapis.com/v1/images:annotate?key={}", api_key);
    let client = Client::new();

    let request_body = VisionRequest {
        requests: vec![
            Request {
                image: Image {
                    content: image_base64.to_string(),
                },
                features: vec![
                    Feature {
                        type_: "LABEL_DETECTION".to_string(),
                    },
                ],
            },
        ],
    };


    let response = client
        .post(&url)
        .json(&request_body)
        .send()?
        .json::<VisionResponse>()?;

        Ok(response)
}

fn image_to_base64(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(general_purpose::STANDARD.encode(&buffer))
}

fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set");
    let image_base64 = image_to_base64("./dog.jpg")?;

    match make_vision_request(&api_key, &image_base64) {
        Ok(response) => {
            for annotation in &response.responses {
                if let Some(labels) = &annotation.labelAnnotations {
                    for label in labels {
                        println!("Etiqueta: {} (Confianza: {:.2})", label.description, label.score);
                    }
                } else {
                    println!("No se encontraron etiquetas");
                }
            }
        }
        Err(err) => {
            eprintln!("Error al hacer la petición: {}", err);
        }
    }

    Ok(())
}
