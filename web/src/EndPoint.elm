module EndPoint exposing (..)

import Config
import Url.Builder exposing (QueryParameter)


type EndPoint
    = Invite
    | Register
    | Auth
    | App_ App


type App
    = Tasks
    | Task Int


url : EndPoint -> List QueryParameter -> String
url ep query =
    let
        path =
            case ep of
                Invite ->
                    [ "invite" ]

                Register ->
                    [ "register" ]

                Auth ->
                    [ "auth" ]

                App_ app ->
                    "app"
                        :: (case app of
                                Tasks ->
                                    [ "tasks" ]

                                Task tid ->
                                    [ "task", String.fromInt tid ]
                           )
    in
    Url.Builder.crossOrigin Config.epBase path query
