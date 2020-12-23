module Div.A exposing (..)

import Html exposing (Html)
import Html.Attributes exposing (id)
import Page as P
import Page.App.App as App
import Page.Invite as Invite
import Page.LP as LP
import Page.Login as Login
import Page.Register as Register
import Util as U



-- MODEL


type Mdl
    = LPMdl LP.Mdl
    | InviteMdl Invite.Mdl
    | RegisterMdl Register.Mdl
    | LoginMdl Login.Mdl
    | AppMdl_ AppMdl


type AppMdl
    = AppMdl App.Mdl


init : ( Mdl, Cmd Msg )
init =
    LP.init |> U.map LPMdl LPMsg



-- UPDATE


type Msg
    = LPMsg LP.Msg
    | InviteMsg Invite.Msg
    | RegisterMsg Register.Msg
    | LoginMsg Login.Msg
    | AppMsg_ AppMsg


type AppMsg
    = AppMsg App.Msg


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case findGoto msg of
        Just page ->
            goto page mdl

        _ ->
            case ( msg, mdl ) of
                ( LPMsg msg_, LPMdl mdl_ ) ->
                    LP.update msg_ mdl_ |> U.map LPMdl LPMsg

                ( InviteMsg msg_, InviteMdl mdl_ ) ->
                    Invite.update msg_ mdl_ |> U.map InviteMdl InviteMsg

                ( RegisterMsg msg_, RegisterMdl mdl_ ) ->
                    Register.update msg_ mdl_ |> U.map RegisterMdl RegisterMsg

                ( LoginMsg msg_, LoginMdl mdl_ ) ->
                    Login.update msg_ mdl_ |> U.map LoginMdl LoginMsg

                ( AppMsg_ (AppMsg msg_), AppMdl_ (AppMdl mdl_) ) ->
                    App.update msg_ mdl_ |> U.map (AppMdl_ << AppMdl) (AppMsg_ << AppMsg)

                _ ->
                    ( mdl, Cmd.none )



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.div [ id "div0" ]
        [ case mdl of
            LPMdl m ->
                LP.view m |> Html.map LPMsg

            InviteMdl m ->
                Invite.view m |> Html.map InviteMsg

            RegisterMdl m ->
                Register.view m |> Html.map RegisterMsg

            LoginMdl m ->
                Login.view m |> Html.map LoginMsg

            AppMdl_ (AppMdl m) ->
                App.view m |> Html.map (AppMsg_ << AppMsg)
        ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    case mdl of
        LPMdl m ->
            LP.subscriptions m |> Sub.map LPMsg

        InviteMdl m ->
            Invite.subscriptions m |> Sub.map InviteMsg

        RegisterMdl m ->
            Register.subscriptions m |> Sub.map RegisterMsg

        LoginMdl m ->
            Login.subscriptions m |> Sub.map LoginMsg

        AppMdl_ (AppMdl m) ->
            App.subscriptions m |> Sub.map (AppMsg_ << AppMsg)



-- HELPER


findGoto : Msg -> Maybe P.Page
findGoto msg =
    case msg of
        LPMsg (LP.Goto page) ->
            Just page

        InviteMsg (Invite.Goto page) ->
            Just page

        RegisterMsg (Register.Goto page) ->
            Just page

        LoginMsg (Login.Goto page) ->
            Just page

        AppMsg_ (AppMsg (App.Goto page)) ->
            Just page

        _ ->
            Nothing


goto : P.Page -> Mdl -> ( Mdl, Cmd Msg )
goto page mdl =
    case page of
        P.LP ->
            LP.init |> U.map LPMdl LPMsg

        P.Invite ->
            case mdl of
                LoginMdl m ->
                    Invite.init m.forgot_pw |> U.map InviteMdl InviteMsg

                _ ->
                    ( mdl, Cmd.none )

        P.Register ->
            case mdl of
                InviteMdl m ->
                    Register.init m.req.email m.req.forgot_pw |> U.map RegisterMdl RegisterMsg

                _ ->
                    ( mdl, Cmd.none )

        P.Login ->
            Login.init |> U.map LoginMdl LoginMsg

        P.App_ P.App ->
            case mdl of
                LPMdl m ->
                    App.init m.user |> U.map (AppMdl_ << AppMdl) (AppMsg_ << AppMsg)

                _ ->
                    ( mdl, Cmd.none )
