module Util exposing (..)

import Config
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onInput)
import Http
import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode
import Task


strEP : EP.EndPoint -> String
strEP ep =
    Config.epBase
        ++ "/"
        ++ (case ep of
                EP.Invite ->
                    "invite"

                EP.Register ->
                    "register"

                EP.Auth ->
                    "auth"

                EP.App ->
                    "app"
           )


type alias HttpResult a =
    Result Http.Error a



-- Http.riskyRequest allows API to set and receive Cookie


get : EP.EndPoint -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
get ep resMsg dec =
    Http.riskyRequest
        { method = "GET"
        , headers = []
        , url = strEP ep
        , body = Http.emptyBody
        , expect = Http.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


post : EP.EndPoint -> Encode.Value -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
post ep enc resMsg dec =
    Http.riskyRequest
        { method = "POST"
        , headers = []
        , url = strEP ep
        , body = Http.jsonBody enc
        , expect = Http.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


post_ : EP.EndPoint -> Encode.Value -> (HttpResult () -> msg) -> Cmd msg
post_ ep enc resMsg =
    Http.riskyRequest
        { method = "POST"
        , headers = []
        , url = strEP ep
        , body = Http.jsonBody enc
        , expect = Http.expectWhatever resMsg
        , timeout = Nothing
        , tracker = Nothing
        }


delete : EP.EndPoint -> (HttpResult () -> msg) -> Cmd msg
delete ep resMsg =
    Http.riskyRequest
        { method = "DELETE"
        , headers = []
        , url = strEP ep
        , body = Http.emptyBody
        , expect = Http.expectWhatever resMsg
        , timeout = Nothing
        , tracker = Nothing
        }


strHttpError : Http.Error -> String
strHttpError e =
    case e of
        Http.BadUrl msg ->
            msg

        Http.Timeout ->
            "Timeout"

        Http.NetworkError ->
            "Network Error"

        -- TODO improve "Bad Status 400" to "Bad Request: invitation expired" etc.
        Http.BadStatus code ->
            "Bad Status " ++ String.fromInt code

        Http.BadBody msg ->
            msg


cmd : (a -> msg) -> a -> Cmd msg
cmd msgFrom x =
    Task.perform msgFrom (Task.succeed x)


map : (a -> mdl) -> (b -> msg) -> ( a, Cmd b ) -> ( mdl, Cmd msg )
map toMdl toMsg =
    Tuple.mapBoth toMdl (Cmd.map toMsg)


input : String -> String -> String -> (String -> msg) -> Html msg
input t p v toMsg =
    Html.input [ type_ t, placeholder p, value v, onInput toMsg ] []
