module Page.LP exposing (..)

import Dict
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class)
import Http
import Json.Decode as Decode exposing (Decoder, string)
import Json.Decode.Pipeline exposing (required)
import Page as P
import Time
import TimeZone
import Util as U



-- MODEL


type alias Mdl =
    { user : User
    , msg : String
    }


type alias User =
    { name : String
    , zone : Time.Zone
    , timescale : U.Timescale
    }


type alias Res =
    { name : String
    , tz : String
    , timescale : String
    }


init : ( Mdl, Cmd Msg )
init =
    ( { user = { name = "", zone = Time.utc, timescale = U.timescale "D" }
      , msg = ""
      }
    , getMe
    )


getMe : Cmd Msg
getMe =
    U.get EP.Auth [] (FromS << GotYou) decRes


decRes : Decoder Res
decRes =
    Decode.succeed Res
        |> required "name" string
        |> required "tz" string
        |> required "timescale" string



-- UPDATE


type Msg
    = Goto P.Page
    | FromS FromS


type FromS
    = GotYou (U.HttpResult Res)


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        Goto _ ->
            ( mdl, Cmd.none )

        FromS fromS ->
            case fromS of
                GotYou (Err e) ->
                    case U.errCode e of
                        Just 401 ->
                            -- Unauthorized
                            ( mdl, U.cmd Goto P.Login )

                        _ ->
                            ( { mdl | msg = U.strHttpError e }, Cmd.none )

                GotYou (Ok ( _, res )) ->
                    ( { mdl
                        | user =
                            { name = res.name
                            , zone =
                                Dict.get res.tz TimeZone.zones
                                    |> Maybe.map (\f -> f ())
                                    |> Maybe.withDefault Time.utc
                            , timescale = U.timescale res.timescale
                            }
                      }
                    , U.cmd Goto (P.App_ P.App)
                    )



-- VIEW


view : Mdl -> Html Msg
view mdl =
    div [ class "pre-app" ]
        [ h1 [ class "pre-app__title" ] [ text "LP" ]
        , div [] [ text mdl.msg ]
        ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
