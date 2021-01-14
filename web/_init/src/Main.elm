module Main exposing (main)

import Browser
import Div.A as Div0
import Div.B as Div1
import Html exposing (Html)
import Html.Attributes exposing (class)


main : Program () Mdl Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Mdl =
    { div0 : Div0.Mdl
    , div1 : Div1.Mdl
    }


init : () -> ( Mdl, Cmd Msg )
init _ =
    let
        ( m0, c0 ) =
            Div0.init

        ( m1, c1 ) =
            Div1.init
    in
    ( Mdl m0 m1
    , Cmd.batch
        [ c0 |> Cmd.map Msg0
        , c1 |> Cmd.map Msg1
        ]
    )



-- UPDATE


type Msg
    = Msg0 Div0.Msg
    | Msg1 Div1.Msg


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        Msg0 msg0 ->
            let
                ( m0, c0 ) =
                    Div0.update msg0 mdl.div0
            in
            ( { mdl | div0 = m0 }, c0 |> Cmd.map Msg0 )

        Msg1 msg1 ->
            let
                ( m1, c1 ) =
                    Div1.update msg1 mdl.div1
            in
            ( { mdl | div1 = m1 }, c1 |> Cmd.map Msg1 )



-- VIEW


view : Mdl -> Html Msg
view mdl =
    Html.div [ class "univ" ]
        [ Div0.view mdl.div0 |> Html.map Msg0
        , Div1.view mdl.div1 |> Html.map Msg1
        ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.batch
        [ Div0.subscriptions mdl.div0 |> Sub.map Msg0
        , Div1.subscriptions mdl.div1 |> Sub.map Msg1
        ]



-- HELPER
