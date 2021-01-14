module Page.Invite exposing (..)

import Bool.Extra as BX
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import Json.Encode as Encode
import Page as P
import Task
import Time
import Util as U



-- MODEL


type alias Mdl =
    { req : Req
    , msg : String
    }


type alias Req =
    { email : String
    , forgot_pw : Bool
    , tz : String
    }


init : Bool -> ( Mdl, Cmd Msg )
init forgot_pw =
    ( { req = { email = "", forgot_pw = forgot_pw, tz = "" }
      , msg = ""
      }
    , Task.perform SetTz Time.getZoneName
    )



-- UPDATE


type Msg
    = Goto P.Page
    | SetTz Time.ZoneName
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
        Goto _ ->
            ( mdl, Cmd.none )

        SetTz zoneName ->
            let
                req =
                    mdl.req

                newReq =
                    { req
                        | tz =
                            case zoneName of
                                Time.Name name ->
                                    name

                                _ ->
                                    "UTC"
                    }
            in
            ( { mdl | req = newReq }, Cmd.none )

        FromU fromU ->
            case fromU of
                InviteMe ->
                    ( mdl, U.post_ EP.Invite (enc mdl.req) (FromS << InvitedYou) )

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


enc : Req -> Encode.Value
enc req =
    Encode.object
        [ ( "email", Encode.string req.email )
        , ( "forgot_pw", Encode.bool req.forgot_pw )
        , ( "tz", Encode.string req.tz )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    div [ class "pre-app" ]
        [ div [ class "pre-app__title" ] [ mdl.req.forgot_pw |> BX.ifElse "Forgot Password" "Invite" |> text ]
        , div [] [ U.input "email" "Email" mdl.req.email EditEmail ]
        , div [] [ button [ onClick InviteMe ] [ mdl.req.forgot_pw |> BX.ifElse "Get Reset Key" "Get Invitation" |> text ] ]
        , div [] [ text mdl.msg ]
        ]
        |> Html.map FromU



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
