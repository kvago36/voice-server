use crate::api::audio_format_options::AudioFormat;
use crate::api::eou_classifier_options::Classifier;
use crate::api::language_restriction_options::LanguageRestrictionType;
use crate::api::raw_audio::AudioEncoding;
use crate::api::recognition_model_options::AudioProcessingType;
use crate::api::recognizer_client::RecognizerClient;
use crate::api::streaming_response::Event;
use crate::api::text_normalization_options::{PhoneFormattingMode, TextNormalization};
use crate::api::{
    AudioChunk, AudioFormatOptions, Eou, EouClassifierOptions, ExternalEouClassifier,
    LanguageRestrictionOptions, RawAudio, RecognitionModelOptions, TextNormalizationOptions,
};
use crate::api::{StreamingOptions, StreamingRequest, streaming_request};

use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, error::Error, rt, web};
use actix_ws::Message;
use futures_util::{SinkExt, StreamExt as _};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::service::Interceptor;
use tonic::transport::Body;
use tonic::{Status};

use crate::state::State;

struct MyInterceptor {
    pub folder_id: String,
    pub token: String,
}

impl Interceptor for MyInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        request
            .metadata_mut()
            .insert("x-folder-id", self.folder_id.parse().unwrap());

        Ok(request)
    }
}

pub async fn handle_message(
    data: web::Data<State>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    let channel = tonic::transport::Endpoint::new("https://stt.api.cloud.yandex.net:443")
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = RecognizerClient::with_interceptor(
        channel,
        MyInterceptor {
            folder_id: data.folder_id.clone(),
            token: data.token.clone(),
        },
    );

    let options = StreamingOptions {
        recognition_model: Some(RecognitionModelOptions {
            model: "general".to_string(),
            audio_format: Some(AudioFormatOptions {
                audio_format: Some(AudioFormat::RawAudio(RawAudio {
                    audio_encoding: AudioEncoding::Linear16Pcm.into(),
                    sample_rate_hertz: 8000,
                    audio_channel_count: 1,
                })),
            }),
            text_normalization: Some(TextNormalizationOptions {
                text_normalization: TextNormalization::Enabled.into(),
                literature_text: false,
                profanity_filter: false,
                phone_formatting_mode: PhoneFormattingMode::Unspecified.into(),
            }),
            language_restriction: Some(LanguageRestrictionOptions {
                language_code: vec!["ru-RU".to_string()],
                restriction_type: LanguageRestrictionType::Whitelist.into(),
            }),
            audio_processing_type: AudioProcessingType::RealTime.into(),
        }),
        eou_classifier: Some(EouClassifierOptions {
            classifier: Some(Classifier::ExternalClassifier(ExternalEouClassifier {})),
        }),
        recognition_classifier: None,
        speech_analysis: None,
        speaker_labeling: None,
    };

    let (mut event_tx, event_rx) = mpsc::channel(12);
    let (mut response_tx, mut response_rx) = mpsc::channel(12);
    let events_stream = ReceiverStream::new(event_rx);

    // TODO: catch error
    event_tx
        .send(StreamingRequest {
            event: Some(streaming_request::Event::SessionOptions(options)),
        })
        .await
        .unwrap();

    rt::spawn(async move {
        let mut response = client.recognize_streaming(events_stream).await.unwrap();

        let mut srv_stream = response.into_inner();

        while let Some(message) = srv_stream.next().await {
            // println!("Got a message: {:?}", message);
            if let Ok(s) = message {
                if let Some(event) = s.event {
                    match event {
                        Event::Partial(fragments) => {
                            if fragments.alternatives.len() > 0 {
                                let text = fragments
                                    .alternatives
                                    .iter()
                                    .fold(String::new(), |acc, a| acc + &a.text);

                                response_tx.send(text).await.unwrap();
                            }
                        }
                        Event::Final(result) => {
                            if result.alternatives.len() > 0 {
                                let text = result
                                    .alternatives
                                    .iter()
                                    .fold(String::new(), |acc, a| acc + &a.text);

                                response_tx.send(text).await.unwrap();
                            }
                        }
                        Event::EouUpdate(end) => {
                            println!("EouUpdate: {:?}", end);
                        }
                        Event::FinalRefinement(refinement) => {
                            println!("FinalRefinement: {:?}", refinement);
                        }
                        Event::StatusCode(_) => {}
                        Event::ClassifierUpdate(_) => {}
                        Event::SpeakerAnalysis(_) => {}
                        Event::ConversationAnalysis(_) => {}
                    }
                } else {
                    println!("event: {:?}", s);
                }
            } else {
                println!("error: {:?}", message);
            }
        }

        println!("done");
    });

    rt::spawn(async move {
        while let Some(result) = response_rx.recv().await {
            session.text(result).await.unwrap();
        }
    });

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Text(_text)) => {
                    let eof_request = StreamingRequest {
                        event: Some(streaming_request::Event::Eou(Eou {})),
                    };

                    event_tx.send(eof_request).await.unwrap();
                }

                Ok(Message::Binary(audio)) => {
                    println!("Got binary len: {:?}", audio.len());

                    let audio_chunk = AudioChunk {
                        data: audio.to_vec(),
                    };

                    let audio_request = StreamingRequest {
                        event: Some(streaming_request::Event::Chunk(audio_chunk)),
                    };

                    event_tx.send(audio_request).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
