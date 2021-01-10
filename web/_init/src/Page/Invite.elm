module Page.Invite exposing (..)

import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import Json.Encode as Encode
import Page as P
import Util as U



-- MODEL


type alias Mdl =
    { req : Req
    , msg : String
    }


type alias Req =
    { email : String
    , forgot_pw : Bool
    }


init : Bool -> ( Mdl, Cmd Msg )
init forgot_pw =
    ( { req = { email = "", forgot_pw = forgot_pw }, msg = "" }, Cmd.none )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = InviteMe
    | EditEmail String


type FromS
    = InvitedYou U.HttpResultAny


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromU fromU ->
            case fromU of
                InviteMe ->
                    ( mdl, U.post_ EP.Invite (encReq mdl.req) (FromS << InvitedYou) )

                EditEmail s ->
                    let
                        req =
                            mdl.req

                        newReq =
                            { req | email = s }
                    in
                    ( { mdl | req = newReq }, Cmd.none )

        FromS fromS ->
            case fromS of
                InvitedYou (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                InvitedYou (Ok _) ->
                    ( mdl, U.cmd Goto P.Register )

        _ ->
            ( mdl, Cmd.none )


encReq : Req -> Encode.Value
encReq req =
    Encode.object
        [ ( "email", Encode.string req.email )
        , ( "forgot_pw", Encode.bool req.forgot_pw )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.map FromU <|
        div []
            [ div [ class "title" ]
                [ text
                    (if mdl.req.forgot_pw then
                        "Forgot Password"

                     else
                        "Invite"
                    )
                ]
            , div [] [ U.input "email" "Email" mdl.req.email EditEmail ]
            , div []
                [ button [ onClick InviteMe ]
                    [ text
                        (if mdl.req.forgot_pw then
                            "Get Reset Key"

                         else
                            "Get Invitation"
                        )
                    ]
                ]
            , div [] [ text mdl.msg ]
            ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
