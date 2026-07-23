// use crate::cmd;
// use waproto::whatsapp::{self as wa, message::interactive_message::Footer};
// use whatsapp_rust::{NodeBuilder, SendOptions};
// use wa::message::interactive_message::{
//     Body, Header, InteractiveMessage, NativeFlowMessage, native_flow_message::NativeFlowButton,
// };

// cmd!(
//     NativeTest,
//     name: "nativeflow",
//     aliases: ["ntv", "beton"],
//     category: "testing",
//     execute: |ctx| {
//         let buttons = vec![
//             NativeFlowButton {
//                 name: Some("quick_reply".to_string()),
//                 button_params_json: Some(r#"{"display_text":"Quick Reply","id":"ngentot"}"#.to_string()),
//             },
//             NativeFlowButton {
//                 name: Some("cta_copy".to_string()),
//                 button_params_json: Some(r#"{"display_text":"Copy Code","copy_code":"kopi"}"#.to_string()),
//             },
//             NativeFlowButton {
//                 name: Some("cta_url".to_string()),
//                 button_params_json: Some(r#"{"display_text":"Webview","url":"https://example.com","webview_interaction":true}"#.to_string()),
//             },
//             NativeFlowButton {
//                 name: Some("single_select".to_string()),
//                 button_params_json: Some(r#"{
//                     "title": "Select Category",
//                     "sections": [
//                         {
//                             "title": "Main Menu",
//                             "rows": [
//                                 {
//                                     "id": "menu_1",
//                                     "title": "Nyawit",
//                                     "description": "afjsa"
//                                 },
//                                 {
//                                     "id": "menu_2",
//                                     "title": "Memek",
//                                     "description": "asfawrfvcazfv"
//                                 }
//                             ]
//                         },
//                         {
//                             "title": "kentot",
//                             "rows": [
//                                 {
//                                     "id": "menu_3",
//                                     "title": "acdnoancs",
//                                     "description": "kanjut"
//                                 }
//                             ]
//                         }
//                     ]
//                 }"#.to_string()),
//             },
//         ];

//         let interactive = wa::message::InteractiveMessage {
//             header: Some(Box::new(Header {
//                 title: Some("test".to_string()),
//                 ..Default::default()
//             })),
//             body: Some(Body {
//                 text: Some("Hello!\nYour message here...".to_string())
//             }),
//             footer: Some(Box::new(Footer {
//                 text: Some("This is Footer".to_string()),
//                 ..Default::default()
//             })),
//             interactive_message: Some(InteractiveMessage::NativeFlowMessage(NativeFlowMessage {
//                 buttons,
//                 message_version: Some(1),
//                 message_params_json: None,
//             })),
//             ..Default::default()
//         };

//         let msg = wa::Message {
//             document_with_caption_message: Some(Box::new(wa::message::FutureProofMessage {
//                 message: Some(Box::new(wa::Message {
//                     interactive_message: Some(Box::new(interactive)),
//                     ..Default::default()
//                 })),
//             })),
//             ..Default::default()
//         };

//         let biz_node = NodeBuilder::new("biz")
//             .children([NodeBuilder::new("interactive")
//                 .attr("type", "native_flow")
//                 .attr("v", "1")
//                 .children([NodeBuilder::new("native_flow")
//                     .attr("v", "9")
//                     .attr("name", "mixed")
//                     .build()])
//                 .build()])
//             .build();
//         println!("{:#?}", biz_node);
//         let options = SendOptions {
//             extra_stanza_nodes: vec![biz_node]
//         };
        
//         ctx.client.send_message_with_options(ctx.info.source.chat.clone(), msg, options).await?;
//     }
// );
