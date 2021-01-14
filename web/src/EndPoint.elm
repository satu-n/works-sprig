module EndPoint exposing (..)


type EndPoint
    = Invite
    | Register
    | Auth
    | App_ App


type App
    = Tasks
    | Task Int
