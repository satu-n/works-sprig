use sparkpost::transmission;

use crate::models;
use crate::errors;
use crate::utils;

lazy_static::lazy_static! { // TODO lazy_static?
    static ref API_KEY: String = utils::env_var("SPARKPOST_API_KEY");
}

pub fn send(invitation: &models::Invitation) -> Result<(), errors::ServiceError> {
    let tm = transmission::Transmission::new(API_KEY.as_str());
    let sending_addr = utils::env_var("SENDING_EMAIL_ADDRESS");
    let sender = utils::env_var("APP_NAME");
    let mut email = transmission::Message::new(
        transmission::EmailAddress::new(sending_addr, sender)
    );
    let recipient: transmission::Recipient = invitation.email.as_str().into();
    let subject = if invitation.forgot_pw {
        "Password reset information".into()
    } else {
        format!("Invitation to {}", utils::env_var("APP_NAME"))
    };
    let body = format!("\
        Your {} key is: <br>
        <span style=\"font-size: x-large; font-weight: bold;\">{}</span> <br>
        The key expires on: <br>
        <span style=\"font-weight: bold;\">{}</span> <br>
        ",
        if invitation.forgot_pw {"reset"} else {"register"},
        invitation.id,
        invitation.expires_at
            .format("%Y-%m-%dT%H:%M:%S%:z") // RFC 3339
            .to_string()
    );
    email
        .add_recipient(recipient)
        // .options(options)
        .subject(subject)
        .html(body);

    let res = tm.send(&email);

    // note that we only print out the error response from email api
    match res {
        Ok(tm_resp) => match tm_resp {
            transmission::TransmissionResponse::ApiResponse(resp) => {
                println!("SparkPost Response:\n{:#?}", resp);
                Ok(())
            },
            transmission::TransmissionResponse::ApiError(errors) => {
                println!("SparkPost Errors:\n{:#?}", &errors);
                Err(errors::ServiceError::InternalServerError)
            },
        },
        Err(req_err) => {
            println!("SparkPost Request Error:\n{:#?}", req_err);
            Err(errors::ServiceError::InternalServerError)
        },
    }
}
