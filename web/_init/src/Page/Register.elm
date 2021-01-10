module Page.Register exposing (..)

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
    , confirmation : String
    , msg : String
    }


type alias Req =
    { key : String
    , email : String
    , password : String
    , reset_pw : Bool
    }


init : String -> Bool -> ( Mdl, Cmd Msg )
init email reset_pw =
    ( { req = { key = "", email = email, password = "", reset_pw = reset_pw }
      , confirmation = ""
      , msg = ""
      }
    , Cmd.none
    )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = RegisterMe
    | EditKey String
    | EditPassWord String
    | EditConfirmation String


type FromS
    = RegisteredYou U.HttpResultAny


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromU fromU ->
            case fromU of
                RegisterMe ->
                    case faultOf mdl of
                        Just fault ->
                            ( { mdl | msg = fault }, Cmd.none )

                        _ ->
                            ( mdl, U.post_ EP.Register (encReq mdl.req) (FromS << RegisteredYou) )

                EditKey s ->
                    let
                        req =
                            mdl.req

                        newReq =
                            { req | key = s }
                    in
                    ( { mdl | req = newReq }, Cmd.none )

                EditPassWord s ->
                    let
                        req =
                            mdl.req

                        newReq =
                            { req | password = s }
                    in
                    ( { mdl | req = newReq }, Cmd.none )

                EditConfirmation s ->
                    ( { mdl | confirmation = s }, Cmd.none )

        FromS fromS ->
            case fromS of
                RegisteredYou (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                RegisteredYou (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )

        _ ->
            ( mdl, Cmd.none )


faultOf : Mdl -> Maybe String
faultOf mdl =
    if String.length mdl.req.key /= 36 then
        Just
            ("Enter the "
                ++ (if mdl.req.reset_pw then
                        "reset"

                    else
                        "register"
                   )
                ++ " key correctly"
            )

    else if String.length mdl.req.password < 8 then
        Just "Password should be at least 8 length"

    else if mdl.req.password /= mdl.confirmation then
        Just "Password mismatched"

    else
        Nothing


encReq : Req -> Encode.Value
encReq req =
    Encode.object
        [ ( "key", Encode.string req.key )
        , ( "email", Encode.string req.email )
        , ( "password", Encode.string req.password )
        , ( "reset_pw", Encode.bool req.reset_pw )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.map FromU <|
        div []
            [ div [ class "title" ]
                [ text
                    (if mdl.req.reset_pw then
                        "Reset Password"

                     else
                        "Register"
                    )
                ]
            , div []
                [ U.input "password"
                    (if mdl.req.reset_pw then
                        "Reset Key"

                     else
                        "Register Key"
                    )
                    mdl.req.key
                    EditKey
                ]
            , div [] [ U.input "password" "New Password" mdl.req.password EditPassWord ]
            , div [] [ U.input "password" "Confirmation" mdl.confirmation EditConfirmation ]
            , div []
                [ button [ onClick RegisterMe ]
                    [ text
                        (if mdl.req.reset_pw then
                            "Reset Password"

                         else
                            "Register"
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
