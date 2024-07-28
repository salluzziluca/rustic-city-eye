use base64::{engine::general_purpose, Engine};
use reqwest::blocking::Client;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Read},
};

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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ImageClassifier {
    /// Este sera el url sobre el cual el clasificador hara peticiones a
    /// Google Cloud Vision.
    url: String,

    /// Contiene las keywords con las cuales se detectaran incidentes(al etiquetar una imagen, se evaluara por coincidencias
    /// con los elementos de este vector para decidir si la imagen contiene un incidente o no).
    incident_keywords: Vec<String>,
}

impl ImageClassifier {
    /// Se le debe brindar el url para clasificar imagenes con Google Cloud Vision.
    /// Por ejemplo: "https://vision.googleapis.com/v1/images:annotate".
    ///
    /// En caso de no tener configurada la variable de entorno GOOGLE_API_KEY
    /// (la cual debe ser una clave de la API del proyecto en gcloud), se producira un error.
    pub fn new(
        url: String,
        incident_keywords_file_path: &str,
    ) -> Result<ImageClassifier, AnnotationError> {
        let api_key = match env::var("GOOGLE_API_KEY") {
            Ok(key) => key,
            Err(e) => return Err(AnnotationError::ApiKeyNotSet(e.to_string())),
        };

        let url = url + "?key=" + &api_key;
        let incident_keywords = Vec::new();

        let mut classifier = ImageClassifier {
            url,
            incident_keywords,
        };

        classifier.set_incident_keywords(incident_keywords_file_path)?;

        Ok(classifier)
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
                let incident_labels = self.process_annotations(labels);
                result.extend(incident_labels);
            } else {
                println!("No se encontraron etiquetas para la imagen");
            }
        }

        Ok(result)
    }

    /// A partir del resultado de la clasificacion de una imagen, determina si se detecta un incidente o no.
    pub fn annotate_image(&self, image_path: &str) -> Result<bool, AnnotationError> {
        let classify_result = self.classify_image(image_path)?;

        if classify_result.is_empty() {
            return Ok(false);
        }

        Ok(true)
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
        let client = Client::new();

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
                        maxResults: Some(10),
                    },
                ],
            }],
        };

        let response = client.post(&self.url).json(&request_body);

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

    /// Lee el archivo de keywords de incidentes, y setea las lecturas sobre
    /// el campo de keywords que guarda el classifier.
    fn set_incident_keywords(&mut self, path: &str) -> Result<(), AnnotationError> {
        let reader = match File::open(path) {
            Ok(f) => BufReader::new(f),
            Err(e) => return Err(AnnotationError::KeywordsFileError(e.to_string())),
        };

        let mut lines = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(l) => lines.push(l),
                Err(e) => return Err(AnnotationError::KeywordsFileError(e.to_string())),
            };
        }
        self.incident_keywords = lines;
        Ok(())
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

    /// Dado un vector de etiquetas que se hicieron sobre una imagen, se busca si estas etiquetas contiene
    /// alguna de las palabras clave para definir incidentes(se hace una comparacion contra los elementos
    /// de incident_keywords que contiene el classifier).
    fn process_annotations(&self, annotations: &Vec<EntityAnnotation>) -> Vec<(String, f64)> {
        let mut results = Vec::new();

        for annotation in annotations {
            let description_lower = annotation.description.to_lowercase();

            for keyword in &self.incident_keywords {
                if description_lower.contains(&keyword.to_lowercase()) {
                    results.push((annotation.description.clone(), annotation.score));
                    break;
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01_classifier_annotation_ok() -> Result<(), AnnotationError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;
        let image_path = "./tests/ia_annotation_img/assault.png";

        let annotate_result = classifier.annotate_image(image_path)?;

        assert!(annotate_result);

        Ok(())
    }

    #[test]
    fn test_02_assault_detection_ok() -> Result<(), AnnotationError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        let image_path = "./tests/ia_annotation_img/assault.png";
        let classification_result = classifier.annotate_image(image_path)?;

        assert!(classification_result);

        Ok(())
    }

    #[test]
    fn test_03_fire_detection_ok() -> Result<(), AnnotationError> {
        let image_path_one = "./tests/ia_annotation_img/fire.png";
        let image_path_two = "./tests/ia_annotation_img/fire2.png";
        let image_path_three = "./tests/ia_annotation_img/fire3.png";

        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        let classification_result_one = classifier.annotate_image(image_path_one)?;
        let classification_result_two = classifier.annotate_image(image_path_two)?;
        let classification_result_three = classifier.annotate_image(image_path_three)?;

        assert!(classification_result_one);
        assert!(classification_result_two);
        assert!(classification_result_three);

        Ok(())
    }

    #[test]
    fn test_04_crash_detection_ok() -> Result<(), AnnotationError> {
        let image_path_one = "./tests/ia_annotation_img/crash1.png";
        let image_path_two = "./tests/ia_annotation_img/crash2.png";
        let image_path_three = "./tests/ia_annotation_img/crash3.png";
        let image_path_four = "./tests/ia_annotation_img/crash4.png";

        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        let classification_result_one = classifier.annotate_image(image_path_one)?;
        let classification_result_two = classifier.annotate_image(image_path_two)?;
        let classification_result_three = classifier.annotate_image(image_path_three)?;
        let classification_result_four = classifier.annotate_image(image_path_four)?;

        assert!(classification_result_one);
        assert!(classification_result_two);
        assert!(classification_result_three);
        assert!(classification_result_four);

        Ok(())
    }

    #[test]
    fn test_05_rebellion_detection_ok() -> Result<(), AnnotationError> {
        let image_path_one = "./tests/ia_annotation_img/rebellion.png";
        let image_path_two = "./tests/ia_annotation_img/rebellion2.png";

        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        let classification_result_one = classifier.annotate_image(image_path_one)?;
        let classification_result_two = classifier.annotate_image(image_path_two)?;

        assert!(classification_result_one);
        assert!(classification_result_two);

        Ok(())
    }

    #[test]
    fn test_06_no_incident_found_ok() -> Result<(), AnnotationError> {
        let image_path_one = "./tests/ia_annotation_img/no_incident.png";
        let image_path_two = "./tests/ia_annotation_img/no_incident2.png";
        let image_path_three = "./tests/ia_annotation_img/no_incident3.png";

        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        let classification_result_one = classifier.annotate_image(image_path_one)?;
        let classification_result_two = classifier.annotate_image(image_path_two)?;
        let classification_result_three = classifier.annotate_image(image_path_three)?;

        assert!(!classification_result_one);
        assert!(!classification_result_two);
        assert!(!classification_result_three);

        Ok(())
    }

    #[test]
    fn test_07_reading_incident_keywords_ok() -> Result<(), AnnotationError> {
        let url = "https://vision.googleapis.com/v1/images:annotate".to_string();
        let incident_keywords_file_path = "./tests/incident_keywords";
        let mut classifier = ImageClassifier::new(url, incident_keywords_file_path)?;

        classifier.set_incident_keywords(incident_keywords_file_path)?;
        let expected_keywords = vec![
            "flood".to_string(),
            "fire".to_string(),
            "smoke".to_string(),
            "fight".to_string(),
            "accident".to_string(),
            "theft".to_string(),
            "crash".to_string(),
            "gun".to_string(),
            "collision".to_string(),
            "rebellion".to_string(),
            "explosion".to_string(),
            "gas".to_string(),
            "atmospheric phenomenon".to_string(),
            "flame".to_string(),
            "wildfire".to_string(),
        ];

        assert_eq!(classifier.incident_keywords, expected_keywords);

        Ok(())
    }
}
