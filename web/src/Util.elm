module Util exposing (..)

import Bool.Extra as BX
import Date
import Dict
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (classList, placeholder, type_, value)
import Html.Events exposing (onInput)
import Http
import Http.Detailed
import Json.Decode as Decode exposing (Decoder)
import Json.Decode.Pipeline exposing (required)
import Json.Encode as Encode
import List.Extra as LX
import Maybe.Extra as MX
import String.Extra as SX
import Task
import Time
import Time.Extra exposing (Interval(..))
import Url.Builder exposing (QueryParameter)



-- Http.Detailed.Error to inform user of 400 BadRequest details


type alias HttpError =
    Http.Detailed.Error String


type alias HttpResult a =
    Result HttpError ( Http.Metadata, a )


type alias HttpResultAny =
    HttpResult String



-- Http.riskyRequest allows API to set and receive Cookie


request : String -> EP.EndPoint -> List QueryParameter -> Http.Body -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
request method ep query body resMsg dec =
    Http.riskyRequest
        { method = method
        , headers = []
        , url = EP.url ep query
        , body = body
        , expect = Http.Detailed.expectJson resMsg dec
        , timeout = Nothing
        , tracker = Nothing
        }


request_ : String -> EP.EndPoint -> List QueryParameter -> Http.Body -> (HttpResultAny -> msg) -> Cmd msg
request_ method ep query body resMsg =
    Http.riskyRequest
        { method = method
        , headers = []
        , url = EP.url ep query
        , body = body
        , expect = Http.Detailed.expectString resMsg
        , timeout = Nothing
        , tracker = Nothing
        }


get : EP.EndPoint -> List QueryParameter -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
get ep query resMsg dec =
    request "GET" ep query Http.emptyBody resMsg dec


post : EP.EndPoint -> Encode.Value -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
post ep enc resMsg dec =
    request "POST" ep [] (Http.jsonBody enc) resMsg dec


post_ : EP.EndPoint -> Encode.Value -> (HttpResult String -> msg) -> Cmd msg
post_ ep enc resMsg =
    request_ "POST" ep [] (Http.jsonBody enc) resMsg


put : EP.EndPoint -> Encode.Value -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
put ep enc resMsg dec =
    request "PUT" ep [] (Http.jsonBody enc) resMsg dec


put_ : EP.EndPoint -> (HttpResult String -> msg) -> Cmd msg
put_ ep resMsg =
    request_ "PUT" ep [] Http.emptyBody resMsg


delete : EP.EndPoint -> Encode.Value -> (HttpResult a -> msg) -> Decoder a -> Cmd msg
delete ep enc resMsg dec =
    request "DELETE" ep [] (Http.jsonBody enc) resMsg dec


delete_ : EP.EndPoint -> (HttpResult String -> msg) -> Cmd msg
delete_ ep resMsg =
    request_ "DELETE" ep [] Http.emptyBody resMsg


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
                    "Authentication failed."

                500 ->
                    "Internal Server Error"

                code ->
                    [ String.fromInt code, meta.statusText ] |> String.join " "

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


len : List a -> String
len l =
    List.length l |> String.fromInt


len1 : List a -> String
len1 l =
    List.length l
        |> (\len_ -> 0 < len_ |> BX.ifElse (String.fromInt len_) "")


type alias Timescale =
    { interval : Interval
    , multiple : Int
    }


timescale : String -> Timescale
timescale s =
    case s of
        "Y" ->
            Timescale Year 1

        "Q" ->
            Timescale Quarter 1

        "M" ->
            Timescale Month 1

        "W" ->
            Timescale Week 1

        "D" ->
            Timescale Day 1

        "6h" ->
            Timescale Hour 6

        "h" ->
            Timescale Hour 1

        "15m" ->
            Timescale Minute 15

        "m" ->
            Timescale Minute 1

        "s" ->
            Timescale Second 1

        _ ->
            Timescale Day 1


strInterval : Interval -> String
strInterval i =
    case i of
        Year ->
            "Y"

        Quarter ->
            "Q"

        Month ->
            "M"

        Week ->
            "W"

        Day ->
            "D"

        Hour ->
            "h"

        Minute ->
            "m"

        Second ->
            "s"

        _ ->
            "?"


strTimescale : Timescale -> String
strTimescale t =
    1 < t.multiple |> BX.ifElse (t.multiple |> int) "" |> (\mult -> mult ++ strInterval t.interval)


fmtDT : Timescale -> String
fmtDT t =
    case t.interval of
        Year ->
            "Y"

        Quarter ->
            "Y/M"

        Month ->
            "Y/M"

        Week ->
            "M/D"

        Day ->
            "M/D"

        Hour ->
            "/D h:"

        Minute ->
            "h:m"

        Second ->
            "m's"

        _ ->
            "?"


clock : Time.Zone -> Time.Posix -> String
clock z t =
    let
        date =
            Date.fromPosix z t |> Date.format "yyyy/MM/dd EEE "

        time =
            [ Time.toHour
            , Time.toMinute
            ]
                |> List.map
                    (\to -> to z t |> String.fromInt |> String.padLeft 2 '0')
                |> String.join ":"
    in
    date ++ time


strDT : Timescale -> Time.Zone -> Time.Posix -> String
strDT ts z t =
    let
        date =
            Date.fromPosix z t

        h =
            Time.toHour z t |> int |> String.padLeft 2 '0'

        m =
            Time.toMinute z t |> int |> String.padLeft 2 '0'

        s =
            Time.toSecond z t |> int |> String.padLeft 2 '0'
    in
    case ts.interval of
        Year ->
            date |> Date.format "yyyy"

        Quarter ->
            date |> Date.format "yyyy/MM"

        Month ->
            date |> Date.format "yyyy/MM"

        Week ->
            date |> Date.format "MM/dd"

        Day ->
            date |> Date.format "MM/dd"

        Hour ->
            date |> Date.format "/dd" |> (\day -> [ day, " ", h, ":" ]) |> String.concat

        Minute ->
            [ h, ":", m ] |> String.concat

        Second ->
            [ m, "'", s ] |> String.concat

        _ ->
            "?"


int : Int -> String
int =
    String.fromInt


lt : Time.Posix -> Time.Posix -> Bool
lt right left =
    Time.posixToMillis left |> (\l -> l < Time.posixToMillis right)


overwrite : a -> List a -> List Bool -> a
overwrite default xs bs =
    LX.zip xs bs
        |> List.foldl (\( x, b ) acc -> b |> BX.ifElse x acc) default


apply : Int -> (a -> a) -> a -> a
apply n f x =
    List.repeat n ()
        |> List.foldl (\_ -> f) x


enumerate : List a -> List ( Int, a )
enumerate =
    List.indexedMap Tuple.pair


bem : String -> String -> List ( String, Bool ) -> Attribute msg
bem block element modifiers =
    let
        be =
            block ++ (element |> String.isEmpty |> BX.ifElse "" ("__" ++ element))
    in
    ( be, True )
        :: (modifiers |> List.map (Tuple.mapFirst (\m -> be ++ "--" ++ m)))
        |> classList


unconsOr : Char -> String -> Char
unconsOr default s =
    s |> String.uncons |> Maybe.map Tuple.first |> Maybe.withDefault default


idBy : String -> String -> String
idBy block elem =
    block ++ "__" ++ elem


signedDecimal : Int -> Float -> String
signedDecimal n x =
    0 < x |> BX.ifElse "+" "" |> (\sign -> sign ++ decimal n x)


decimal : Int -> Float -> String
decimal n x =
    x * 10 ^ (n |> toFloat) |> round |> String.fromInt |> (\s -> SX.insertAt "." (String.length s - n) s)


between : comparable -> comparable -> comparable -> Bool
between l r x =
    l < x && x < r


type alias Allocation =
    { open_h : Int
    , open_m : Int
    , hours : Int
    }


decAllocation : Decoder Allocation
decAllocation =
    Decode.succeed Allocation
        |> required "open_h" Decode.int
        |> required "open_m" Decode.int
        |> required "hours" Decode.int
