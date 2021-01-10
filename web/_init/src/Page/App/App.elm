module Page.App.App exposing (..)

import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class, classList, href, id, placeholder, value)
import Html.Events exposing (onBlur, onClick, onFocus, onInput)
import Json.Encode as Encode
import Page as P
import Time
import Util as U



-- MODEL


type alias Mdl =
    { user : User
    , req : String
    , msg : String
    }


type alias User =
    { name : String
    , zone : Time.Zone
    }


init : User -> ( Mdl, Cmd Msg )
init user =
    ( { user = user, req = "", msg = "" }, Cmd.none )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = Logout
    | Send
    | EditText String


type FromS
    = LoggedOut U.HttpResultAny
    | Received U.HttpResultAny


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromU fromU ->
            case fromU of
                Logout ->
                    ( mdl, U.delete_ EP.Auth (FromS << LoggedOut) )

                Send ->
                    ( mdl, U.post_ (EP.App_ EP.Tasks) (enc mdl.req) (FromS << Received) )

                EditText s ->
                    ( { mdl | req = s }, Cmd.none )

        FromS fromS ->
            case fromS of
                LoggedOut (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                LoggedOut (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )

                Received (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                Received (Ok ( _, s )) ->
                    ( { mdl | msg = s }, Cmd.none )

        _ ->
            ( mdl, Cmd.none )


enc : String -> Encode.Value
enc s =
    Encode.object
        [ ( "text", Encode.string s )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.map FromU <|
        div []
            [ div [ class "title" ] [ text "App" ]
            , div [] [ text mdl.user.name ]

            -- , div [] [ U.input "text" "/" mdl.req EditText ]
            , div [ id "input-box" ]
                [ textarea
                    [ id "input-area"
                    , value mdl.req
                    , onInput EditText

                    -- , placeholder placeHolder
                    -- , onFocus TypeOn
                    -- , onBlur TypeOff
                    -- , classList
                    --     [ ( "full-screen", mdl.isFullInput )
                    --     ]
                    ]
                    []
                ]
            , div [] [ button [ onClick Send ] [ text "Send" ] ]
            , div [] [ button [ onClick Logout ] [ text "Logout" ] ]
            , div [] [ text mdl.msg ]
            ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
