module Util exposing (..)

import Config
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onInput)
import Http
import Http.Detailed
import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode
import Task


strEP : EP.EndPoint -> String
strEP ep =
    Config.epBase
        ++ (case ep of
                EP.Invite ->
                    "/invite"

                EP.Register ->
                    "/register"

                EP.Auth ->
                    "/auth"

                EP.App_ app ->
                    "/app"
                        ++ (case app of
                                EP.Tasks ->
                                    "/tasks"

                                EP.Task tid ->
                                    "/task" ++ "/" ++ String.fromInt tid
                           )
           )



-- Http.Detailed.Error to inform user of the body of 400 BadRequest


type alias HttpResult a =
    Result (Http.Detailed.Error String) ( Http.Metadata, a )


type alias HttpResultAny =
    HttpResult String



-- Http.riskyRequest allows API to set and receive Cookie


get : EP.EndPoint -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
get ep resMsg dec =
    Http.riskyRequest
        { method = "GET"
        , headers = []
        , url = strEP ep
        , body = Http.emptyBody
        , expect = Http.Detailed.expectJson resMsg dec
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
        , expect = Http.Detailed.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


post_ : EP.EndPoint -> Encode.Value -> (HttpResult String -> msg) -> Cmd msg
post_ ep enc resMsg =
    Http.riskyRequest
        { method = "POST"
        , headers = []
        , url = strEP ep
        , body = Http.jsonBody enc
        , expect = Http.Detailed.expectString resMsg
        , timeout = Nothing
        , tracker = Nothing
        }


put : EP.EndPoint -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
put ep resMsg dec =
    Http.riskyRequest
        { method = "PUT"
        , headers = []
        , url = strEP ep
        , body = Http.emptyBody
        , expect = Http.Detailed.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


delete : EP.EndPoint -> Encode.Value -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
delete ep enc resMsg dec =
    Http.riskyRequest
        { method = "DELETE"
        , headers = []
        , url = strEP ep
        , body = Http.jsonBody enc
        , expect = Http.Detailed.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


delete_ : EP.EndPoint -> (HttpResult String -> msg) -> Cmd msg
delete_ ep resMsg =
    Http.riskyRequest
        { method = "DELETE"
        , headers = []
        , url = strEP ep
        , body = Http.emptyBody
        , expect = Http.Detailed.expectString resMsg
        , timeout = Nothing
        , tracker = Nothing
        }


errCode : Http.Detailed.Error String -> Maybe Int
errCode e =
    case e of
        Http.Detailed.BadStatus meta _ ->
            Just meta.statusCode

        _ ->
            Nothing


strHttpError : Http.Detailed.Error String -> String
strHttpError e =
    case e of
        Http.Detailed.BadUrl msg ->
            msg

        Http.Detailed.Timeout ->
            "Timeout"

        Http.Detailed.NetworkError ->
            "Network Error"

        Http.Detailed.BadStatus meta body ->
            case meta.statusCode of
                400 ->
                    "Oops, " ++ body

                401 ->
                    "Not authenticated."

                500 ->
                    "Internal Server Error"

                c ->
                    String.fromInt c ++ " " ++ meta.statusText

        Http.Detailed.BadBody _ _ msg ->
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
