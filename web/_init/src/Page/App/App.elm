module Page.App.App exposing (..)

import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import Page as P
import Util as U



-- MODEL


type alias Mdl =
    { user : User
    , msg : String
    }


type alias User =
    { name : String }


init : User -> ( Mdl, Cmd Msg )
init user =
    ( { user = user, msg = "" }, Cmd.none )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = Logout


type FromS
    = LoggedOut (U.HttpResult ())


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromU fromU ->
            case fromU of
                Logout ->
                    ( mdl, logout )

        FromS fromS ->
            case fromS of
                LoggedOut (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                LoggedOut (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )

        _ ->
            ( mdl, Cmd.none )


logout : Cmd Msg
logout =
    U.delete EP.Auth (FromS << LoggedOut)



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.map FromU <|
        div []
            [ div [ class "title" ] [ text "App" ]
            , div [] [ text mdl.user.name ]
            , div [] [ text mdl.msg ]
            , div [] [ button [ onClick Logout ] [ text "Logout" ] ]
            ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
