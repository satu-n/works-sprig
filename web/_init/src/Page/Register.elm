module Page.Register exposing (..)

import Bool.Extra as BX
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
    , confirmation : String
    , msg : String
    }


type alias Req =
    { key : String
    , email : String
    , password : String
    , reset_pw : Bool
    }


init : String -> Bool -> ( Mdl, Cmd Msg )
init email reset_pw =
    ( { req = { key = "", email = email, password = "", reset_pw = reset_pw }
      , confirmation = ""
      , msg = ""
      }
    , Cmd.none
    )



-- UPDATE


type Msg
    = Goto P.Page
    | FromU FromU
    | FromS FromS


type FromU
    = RegisterMe
    | EditKey String
    | EditPassWord String
    | EditConfirmation String


type FromS
    = RegisteredYou U.HttpResultAny


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        Goto _ ->
            ( mdl, Cmd.none )

        FromU fromU ->
            case fromU of
                RegisterMe ->
                    case faultOf mdl of
                        Just fault ->
                            ( { mdl | msg = fault }, Cmd.none )

                        _ ->
                            ( mdl, U.post_ EP.Register (enc mdl.req) (FromS << RegisteredYou) )

                EditKey s ->
                    let
                        req =
                            mdl.req

                        newReq =
                            { req | key = s }
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

                EditConfirmation s ->
                    ( { mdl | confirmation = s }, Cmd.none )

        FromS fromS ->
            case fromS of
                RegisteredYou (Err e) ->
                    ( { mdl | msg = U.strHttpError e }, Cmd.none )

                RegisteredYou (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )


faultOf : Mdl -> Maybe String
faultOf mdl =
    let
        passwordLen =
            8
    in
    [ mdl.req.password /= mdl.confirmation
    , String.length mdl.req.password < passwordLen
    , String.length mdl.req.key /= 36
    ]
        |> U.overwrite Nothing
            ([ "Password does not match confirmation."
             , [ "Password should be at least", U.int passwordLen, "length." ] |> String.join " "
             , [ "Enter the"
               , mdl.req.reset_pw |> BX.ifElse "reset" "register"
               , "key correctly."
               ]
                |> String.join " "
             ]
                |> List.map Just
            )


enc : Req -> Encode.Value
enc req =
    Encode.object
        [ ( "key", Encode.string req.key )
        , ( "email", Encode.string req.email )
        , ( "password", Encode.string req.password )
        , ( "reset_pw", Encode.bool req.reset_pw )
        ]



-- VIEW


view : Mdl -> Html Msg
view mdl =
    div [ class "pre-app" ]
        [ div [ class "pre-app__title" ] [ mdl.req.reset_pw |> BX.ifElse "Reset Password" "Register" |> text ]
        , div [] [ U.input "password" (mdl.req.reset_pw |> BX.ifElse "Reset Key" "Register Key") mdl.req.key EditKey ]
        , div [] [ U.input "password" "New Password" mdl.req.password EditPassWord ]
        , div [] [ U.input "password" "Confirmation" mdl.confirmation EditConfirmation ]
        , div [] [ button [ onClick RegisterMe ] [ mdl.req.reset_pw |> BX.ifElse "Reset Password" "Register" |> text ] ]
        , div [] [ text mdl.msg ]
        ]
        |> Html.map FromU



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.none



-- HELPER
