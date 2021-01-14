module Page.Login exposing (..)

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
    , forgot_pw : Bool
    }


type alias Req =
    { email : String
    , password : String
    , tz : String
    }


init : ( Mdl, Cmd Msg )
init =
    ( { req = { email = "", password = "", tz = "" }
      , msg = ""
      , forgot_pw = False
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
    = Login
    | NewAccount
    | ForgotPW
    | EditEmail String
    | EditPassWord String


type FromS
    = LoggedIn U.HttpResultAny


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
                Login ->
                    ( mdl, U.post_ EP.Auth (enc mdl.req) (FromS << LoggedIn) )

                NewAccount ->
                    ( { mdl | forgot_pw = False }, U.cmd Goto P.Invite )

                ForgotPW ->
                    ( { mdl | forgot_pw = True }, U.cmd Goto P.Invite )

                EditEmail s ->
                    let
                        req =
                            mdl.req

                        newReq =
                            { req | email = s }
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

        FromS fromS ->
            case fromS of
                LoggedIn (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                LoggedIn (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )


enc : Req -> Encode.Value
enc req =
    Encode.object
        [ ( "email", Encode.string req.email )
        , ( "password", Encode.string req.password )
        , ( "tz", Encode.string req.tz )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    div [ class "pre-app" ]
        [ div [ class "pre-app__title" ] [ text "Login" ]
        , div [] [ U.input "email" "Email" mdl.req.email EditEmail ]
        , div [] [ U.input "password" "Password" mdl.req.password EditPassWord ]
        , div [] [ button [ onClick Login ] [ text "Login" ] ]
        , div [] [ button [ onClick NewAccount ] [ text "New Account" ] ]
        , div [] [ button [ onClick ForgotPW ] [ text "Forgot Password" ] ]
        , div [] [ text mdl.msg ]
        ]
        |> Html.map FromU



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
