module Page.LP exposing (..)

import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class)
import Http
import Json.Decode as Decode exposing (Decoder, string)
import Json.Decode.Pipeline exposing (required)
import Page as P
import Util as U



-- MODEL


type alias Mdl =
    { user : User
    , msg : String
    }


type alias User =
    { email : String }


init : ( Mdl, Cmd Msg )
init =
    ( { user = { email = "" }, msg = "" }, getMe )


getMe : Cmd Msg
getMe =
    U.get EP.Auth (FromS << GotYou) decUser


decUser : Decoder User
decUser =
    Decode.succeed User
        |> required "email" string



-- UPDATE


type Msg
    = Goto P.Page
    | FromS FromS


type FromS
    = GotYou (U.HttpResult User)


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromS fromS ->
            case fromS of
                GotYou (Err (Http.BadStatus 401)) ->
                    ( mdl, U.cmd Goto P.Login )

                GotYou (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                GotYou (Ok user) ->
                    ( { mdl | user = user }, U.cmd Goto (P.App_ P.App) )

        _ ->
            ( mdl, Cmd.none )



-- VIEW


view : Mdl -> Html Msg
view mdl =
    div []
        [ div [ class "title" ] [ text "LP" ]
        , div [] [ text mdl.msg ]
        ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
