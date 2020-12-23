module Page.Login exposing (..)

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
    , msg : String
    , forgot_pw : Bool
    }


type alias Req =
    { email : String
    , password : String
    }


init : ( Mdl, Cmd Msg )
init =
    ( { req = { email = "", password = "" }, msg = "", forgot_pw = False }, Cmd.none )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = Login
    | NoAccount
    | ForgotPW
    | EditEmail String
    | EditPassWord String


type FromS
    = LoggedIn (U.HttpResult ())


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        FromU fromU ->
            case fromU of
                Login ->
                    ( mdl, login mdl.req )

                NoAccount ->
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

        _ ->
            ( mdl, Cmd.none )


login : Req -> Cmd Msg
login req =
    U.post_ EP.Auth (encReq req) (FromS << LoggedIn)


encReq : Req -> Encode.Value
encReq req =
    Encode.object
        [ ( "email", Encode.string req.email )
        , ( "password", Encode.string req.password )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.map FromU <|
        div []
            [ div [ class "title" ] [ text "Login" ]
            , div [] [ U.input "email" "Email" mdl.req.email EditEmail ]
            , div [] [ U.input "password" "Password" mdl.req.password EditPassWord ]
            , div [] [ button [ onClick Login ] [ text "Login" ] ]
            , div [] [ button [ onClick NoAccount ] [ text "No Account" ] ]
            , div [] [ button [ onClick ForgotPW ] [ text "Forgot Password" ] ]
            , div [] [ text mdl.msg ]
            ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
