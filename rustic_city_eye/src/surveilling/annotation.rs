use base64::{engine::general_purpose, Engine};
use reqwest::blocking::Client;
use std::{env, fs::File, io::Read};

use serde::{Deserialize, Serialize};

use super::annotation_error::AnnotationError;

#[allow(non_snake_case)]
#[derive(Serialize)]
struct Feature {
    #[serde(rename = "type")]
    type_: String,
    maxResults: Option<i32>,
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

/// Se encarga de clasificar imagenes sirviendose de la API que brinda Google Cloud Vision.
///
/// Tiene una instancia de cliente de reqwest, lo que nos va a permitir manejar peticiones http hacia la IA de forma sincronica.
/// Ademas, se guarda el url sobre el cual se van a enviar las peticiones.
///
/// Se le proporcionan paths a imagenes, luego se encarga de realizar la peticion a la IA, y por ultimo devuelve la respuesta a esa peticion,
/// que en nuestro caso sera etiquetar las imagenes. Tambien se proporciona la estadistica del score obtenido sobre cada una de las etiquetas,
/// de forma tal de medir la confiabilidad de la clasificacion.
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
    pub fn classify_image(&self, image_path: &str) -> Result<Vec<(String, f64)>, AnnotationError> {
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
                let incident_labels = self.process_annotations(labels);
                result.extend(incident_labels);
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
                features: vec![
                    Feature {
                        type_: "LABEL_DETECTION".to_string(),
                        maxResults: Some(50),
                    },
                    Feature {
                        type_: "SAFE_SEARCH_DETECTION".to_string(),
                        maxResults: Some(20),
                    },
                ],
            }],
        };

        let response = self.reqwest_client.post(&self.url).json(&request_body);

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
            Ok(_) => {}
            Err(e) => return Err(AnnotationError::ImageError(e.to_string())),
        };
        Ok(general_purpose::STANDARD.encode(&buffer))
    }

    fn process_annotations(&self, annotations: &Vec<EntityAnnotation>) -> Vec<(String, f64)> {
        let mut results = Vec::new();
        let incident_keywords = vec![
            "flood", "fire", "smoke", "fight", "accident", "theft", "crash", "gun", "collision", "rebellion", "explosion", "gas", "atmospheric phenomenon", "flame", "wildfire"
        ];

        for annotation in annotations {
            let description_lower = annotation.description.to_lowercase();

            for keyword in &incident_keywords {
                if description_lower.contains(&keyword.to_lowercase()) {
                    results.push((annotation.description.clone(), annotation.score));
                    break;
                }
            }
        }

        println!("{:?} results", results);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_classifier_annotation_ok() -> Result<(), AnnotationError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let image_path = "./tests/ia_annotation_img/image.png";

        let classifier = ImageClassifier::new(url)?;

        classifier.classify_image(image_path)?;

        Ok(())
    }
}
