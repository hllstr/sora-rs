// use whatsapp_rust::Jid;
// use crate::cmd;

// cmd!(
//     Add,
//     name: "add",
//     aliases: ["invite"],
//     category: "group",
//     execute: |ctx| {
//         println!("{:?}", ctx.body);
//         let number: String = ctx.body.chars().filter(|c| c.is_ascii_digit()).collect();
//         let mut participant = format!("{}@s.whatsapp.net", number);
//         if number.is_empty()
//             && let Some(ext_msg) = &ctx.msg.extended_text_message
//                 && let Some(context) = &ext_msg.context_info
//                     && let Some(p) = &context.participant {
//                         participant = p.clone()
//                     }
//         let mut new_members: Vec<Jid> = vec![participant.to_string().parse()?];
//         println!("{:?}", new_members);
//         if new_members[0].is_lid()
//             && let Some(pn) = ctx.client.get_phone_number_from_lid(new_members[0].user_base()).await {
//                 new_members[0] = format!("{}@s.whatsapp.net", pn).parse()?;
//             }
//         if new_members[0].user.is_empty() {
//             ctx.react("❔").await?;
//             return Ok(());
//         };
//         let responses = ctx.client.groups().add_participants(&ctx.info.source.chat, &new_members).await?;

//         for response in responses {
//             match response.error.as_deref() {
//                 Some("401") => ctx.reply("401: unauthorized").await?,
//                 Some("400") => ctx.reply("400: bad-request").await?,
//                 _ => ctx.react("✅").await?,
//             };
            
//             if let Some(error) = &response.error {
//                 println!("Error: {}", error);
//             }
//         }
//     }
// );