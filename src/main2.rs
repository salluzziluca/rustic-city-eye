use google_vision1::{oauth2, hyper, hyper_rustls};
use google_vision1::api::AnnotateImageRequest;
use vision1::oauth2::ServiceAccountKey;
use std::default::Default;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;
extern crate google_vision1 as vision1;
use vision1::Vision;
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() {

    let sa_key_path = env::var("GOOGLE_APPLICATION_CREDENTIALS").expect("GOOGLE_APPLICATION_CREDENTIALS not set");
    let sa_key_file = File::open(sa_key_path).expect("Service account key file not found");
    let sa_key: ServiceAccountKey = serde_json::from_reader(BufReader::new(sa_key_file)).expect("Invalid service account key file");

    let auth = oauth2::ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await
        .unwrap();

    // Create a new Vision API client
    let hub = Vision::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().unwrap().https_or_http().enable_http1().build()), auth);

    let image_source = google_vision1::api::ImageSource {
        gcs_image_uri: Some("gs://cloud-samples-data/vision/label/wakeupcat.jpg".to_string()),
        image_uri: None
    };

    // Prepare the request
    let req = AnnotateImageRequest {
        image: Some(google_vision1::api::Image {
            source: Some(image_source),
            content: None
        }),
        features: Some(vec![google_vision1::api::Feature {
            type_: Some("LABEL_DETECTION".to_string()),
            max_results: Some(10),
            ..Default::default()
        }]),
        ..Default::default()
    };


    let mut file = File::open("./dog.jpg").expect("Failed to open image file");
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data).expect("Failed to read image file");
    
    // Encode image data to base64
    let b64 = general_purpose::STANDARD.encode(image_data);
    println!("image data: {:?}", b64[0..10].to_string());

    let req2 = AnnotateImageRequest {
        image: Some(google_vision1::api::Image {
            content: Some(b64.into_bytes()),
            source: None
        }),
        features: Some(vec![google_vision1::api::Feature {
            type_: Some("LABEL_DETECTION".to_string()),
            max_results: Some(10),
            ..Default::default()
        }]),
        image_context: None
    };

    let batch_req = google_vision1::api::BatchAnnotateImagesRequest {
        requests: Some(vec![
            req, 
            req2
            ]),
        ..Default::default()
    };

    // Make the API call
    match hub.images().annotate(batch_req).doit().await {
        Err(e) => println!("Error: {}", e),
        Ok(res) => {
            // println!("Response: {:?}", res);
            for response in res.1.responses.unwrap() {
                match response.label_annotations {
                    Some(label) => {
                        for l in label {
                            println!("Label: {:?}", l.description.unwrap());
                        }},
                    None => println!("No label annotations found"),
                }
            }

        },
    }
}
