use base64::{engine::general_purpose, Engine};
use reqwest::blocking::Client;
use std::{env, fs::File, io::Read};

use serde::{Deserialize, Serialize};

use super::annotation_error::AnnotationError;

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

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AnnotateImageResponse {
    labelAnnotations: Option<Vec<EntityAnnotation>>,
}

#[derive(Deserialize, Debug)]
struct VisionResponse {
    responses: Vec<AnnotateImageResponse>,
}

#[derive(Debug)]
pub struct ImageClassifier {
    ///Este sera el url sobre el cual el clasificador hara peticiones a 
    /// Google Cloud Vision.
    url: String,

    /// Sobre este cliente de reqwest, se hacen peticiones sincronicas a la IA de clasificacion. 
    reqwest_client: Client,
}

impl ImageClassifier {

    /// Se le debe brindar el url para clasificar imagenes con Google Cloud Vision.
    /// Por ejemplo: "https://vision.googleapis.com/v1/images:annotate".
    /// 
    /// En caso de no tener configurada la variable de entorno GOOGLE_API_KEY
    /// (la cual debe ser una clave de la API del proyecto en gcloud), se producira un error.
    pub fn new(url: String) -> Result<ImageClassifier, AnnotationError> {
        let api_key = match env::var("GOOGLE_API_KEY") {
            Ok(key) => key,
            Err(e) => return Err(AnnotationError::ApiKeyNotSet(e.to_string())),
        };

        let url = url + "?key=" + &api_key;
        let client = Client::new();

        Ok(ImageClassifier {
            url,
            reqwest_client: client,
        })
    }

    /// Dado un path a una imagen, se abre y se codifica en Base64. Una vez convertida, se hace
    /// una peticion al servicio de clasificacion de imagenes para obtener etiquetas sobre la misma, ademas 
    /// del score(nivel de confianza de la clasificacion) asociado a las mismas.
    pub fn classify_image(
        &self,
        image_path: &str,
    ) -> Result<Vec<(String, f64)>, AnnotationError> {
        let mut result = Vec::new();

        let image_base64 = self.image_to_base64(image_path)?;
        let res = self.make_google_vision_request(&image_base64)?;

        for annotation in &res.responses {
            if let Some(labels) = &annotation.labelAnnotations {
                for label in labels {
                    println!(
                        "Etiqueta: {} (Confianza: {:.2})",
                        label.description, label.score
                    );
                    result.push((label.description.clone(), label.score));
                }
            } else {
                println!("No se encontraron etiquetas para la imagen");
            }
        }

        Ok(result)
    }

    /// Dada una imagen codificada en Base64, se construye una request(la cual va a tener como feature
    /// a Label Detection de la IA utilizada), y se hara un POST sobre el cliente de reqwest.
    /// 
    /// Una vez con la response a la peticion, se devuelve el vector con las etiquetas junto a sus scores
    /// correspodientes.
    fn make_google_vision_request(
        &self,
        image_base64: &str,
    ) -> Result<VisionResponse, AnnotationError> {
        let request_body = VisionRequest {
            requests: vec![Request {
                image: Image {
                    content: image_base64.to_string(),
                },
                features: vec![Feature {
                    type_: "LABEL_DETECTION".to_string(),
                }],
            }],
        };

        let response = self
            .reqwest_client
            .post(&self.url)
            .json(&request_body);

        let response = match response.send() {
            Ok(res) => res,
            Err(e) => return Err(AnnotationError::RequestError(e.to_string())),
        };

        let response = match response.json::<VisionResponse>() {
            Ok(res) => res,
            Err(e) => return Err(AnnotationError::RequestError(e.to_string())),
        };

        Ok(response)
    }

    /// Dado un path a una imagen, se abre y se codifica en Base64.
    fn image_to_base64(&self, path: &str) -> Result<String, AnnotationError> {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(AnnotationError::ImageError(e.to_string())),
        };
        let mut buffer = Vec::new();

        match file.read_to_end(&mut buffer) {
            Ok(_) => {},
            Err(e) => return Err(AnnotationError::ImageError(e.to_string())),
        };
        Ok(general_purpose::STANDARD.encode(&buffer))
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_classifier_annotation_ok() -> Result<(), AnnotationError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let image_path = "./src/surveilling/accident.png";

        let classifier = ImageClassifier::new(url)?;

        classifier.classify_image(image_path)?;

        Ok(())
    }
}
