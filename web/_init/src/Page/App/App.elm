module Page.App.App exposing (..)

import Bool.Extra as BX
import Browser.Dom as Dom
import Browser.Events as Events
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (class, classList, href, id, placeholder, spellcheck, style, target, value)
import Html.Events exposing (onBlur, onClick, onFocus, onInput)
import Json.Decode as Decode exposing (Decoder, bool, float, int, list, null, nullable, oneOf, string)
import Json.Decode.Extra exposing (datetime)
import Json.Decode.Pipeline exposing (required, requiredAt)
import Json.Encode as Encode
import List.Extra as LX
import Maybe.Extra as MX
import Page as P
import Page.App.Placeholder as Placeholder
import Task
import Time exposing (Posix)
import Time.Extra as TX
import Url.Builder
import Util as U



-- MODEL


type alias Mdl =
    { user : User
    , input : String

    -- TODO , inputLog : List String
    , msg : String
    , items : List Item
    , cursor : Index
    , selected : List Tid
    , timescale : U.Timescale
    , view : View
    , now : Posix
    , asOf : Posix
    , isCurrent : Bool
    , isInput : Bool
    , isInputFS : Bool
    , keyMod : KeyMod
    }


type alias User =
    { name : String
    , zone : Time.Zone
    , timescale : U.Timescale
    }


type alias Index =
    Int


type alias Tid =
    Int


type View
    = Home_
    | Leaves
    | Roots
    | Archives
    | Focus_
    | Search
    | Tutorial


type alias KeyMod =
    { ctrl : Bool
    , shift : Bool
    }


init : User -> ( Mdl, Cmd Msg )
init user =
    ( { user = user
      , input = ""
      , msg = [ "Hello", user.name ] |> String.join " "
      , items = []
      , cursor = 0
      , selected = []
      , timescale = U.timescale "D"
      , view = Home_
      , now = Time.millisToPosix 0
      , asOf = Time.millisToPosix 0
      , isCurrent = True
      , isInput = False
      , isInputFS = False
      , keyMod = KeyMod False False
      }
    , Home { option = Nothing } |> request
    )



-- UPDATE


type Msg
    = Goto P.Page
    | NoOp
    | Tick Posix
    | FromU FromU
    | FromS FromS


type FromU
    = Request Req
    | Input String
    | InputBlur
    | InputFocus
    | KeyDown Key
    | KeyUp Key
    | Select Tid


type FromS
    = LoggedOut U.HttpResultAny
    | Homed (U.HttpResult ResHome)
    | Texted (U.HttpResult ResText)
    | Cloned (U.HttpResult ResClone)
    | Execed (U.HttpResult ResExec)
    | Focused (U.HttpResult ResFocus)
    | Starred U.HttpResultAny


update : Msg -> Mdl -> ( Mdl, Cmd Msg )
update msg mdl =
    case msg of
        Goto _ ->
            ( mdl, Cmd.none )

        NoOp ->
            ( mdl, Cmd.none )

        Tick now ->
            ( { mdl
                | now = now
                , asOf = mdl.isCurrent |> BX.ifElse now mdl.asOf
              }
            , Cmd.none
            )

        FromU fromU ->
            case fromU of
                Request req ->
                    ( mdl, request req )

                Input s ->
                    ( { mdl | input = s }, Cmd.none )

                InputBlur ->
                    ( { mdl | isInput = False }, Cmd.none )

                InputFocus ->
                    ( { mdl | isInput = True }, Cmd.none )

                KeyDown key ->
                    case key of
                        Char c ->
                            mdl.isInput
                                |> BX.ifElse ( mdl, Cmd.none )
                                    (case c of
                                        '/' ->
                                            ( mdl, Dom.focus idInput |> Task.attempt (\_ -> NoOp) )

                                        'w' ->
                                            ( { mdl | asOf = mdl.asOf |> timeshift mdl -1, isCurrent = False }, Cmd.none )

                                        'o' ->
                                            ( { mdl | asOf = mdl.asOf |> timeshift mdl 1, isCurrent = False }, Cmd.none )

                                        'j' ->
                                            ( { mdl | cursor = mdl.cursor |> U.ifTrue (\cur -> cur < List.length mdl.items - 1) ((+) 1) }, follow Down mdl )

                                        'k' ->
                                            ( { mdl | cursor = mdl.cursor |> U.ifTrue (\cur -> 0 < cur) ((-) 1) }, follow Up mdl )

                                        'x' ->
                                            ( mdl, (\item -> Select item.id |> U.cmd FromU) |> forTheItem mdl )

                                        -- TODO port https://stackoverflow.com/questions/65316506/elm-open-url-in-a-new-tab
                                        'u' ->
                                            ( mdl, Cmd.none )

                                        'i' ->
                                            ( { mdl | selected = mdl.items |> List.filter (\item -> LX.notMember item.id mdl.selected) |> List.map .id }, Cmd.none )

                                        's' ->
                                            ( mdl, (\item -> Star item.id |> request) |> forTheItem mdl )

                                        'f' ->
                                            ( mdl, (\item -> Focus item.id |> request) |> forTheItem mdl )

                                        'e' ->
                                            ( mdl, Exec { tids = mdl.selected, revert = mdl.keyMod.shift } |> request )

                                        'c' ->
                                            ( mdl, Clone mdl.selected |> request )

                                        'a' ->
                                            ( mdl, Home { option = Just "archives" } |> request )

                                        'r' ->
                                            ( mdl, Home { option = Just "roots" } |> request )

                                        'l' ->
                                            ( mdl, Home { option = Just "leaves" } |> request )

                                        'h' ->
                                            ( mdl, Home { option = Nothing } |> request )

                                        '1' ->
                                            ( { mdl | timescale = U.timescale "Y" }, Cmd.none )

                                        '2' ->
                                            ( { mdl | timescale = U.timescale "Q" }, Cmd.none )

                                        '3' ->
                                            ( { mdl | timescale = U.timescale "M" }, Cmd.none )

                                        '4' ->
                                            ( { mdl | timescale = U.timescale "W" }, Cmd.none )

                                        '5' ->
                                            ( { mdl | timescale = U.timescale "D" }, Cmd.none )

                                        '6' ->
                                            ( { mdl | timescale = U.timescale "6h" }, Cmd.none )

                                        '7' ->
                                            ( { mdl | timescale = U.timescale "h" }, Cmd.none )

                                        '8' ->
                                            ( { mdl | timescale = U.timescale "15m" }, Cmd.none )

                                        '9' ->
                                            ( { mdl | timescale = U.timescale "m" }, Cmd.none )

                                        _ ->
                                            ( mdl, Cmd.none )
                                    )

                        NonChar nc ->
                            case nc of
                                Modifier m ->
                                    ( { mdl | keyMod = mdl.keyMod |> setKeyMod m True }, Cmd.none )

                                Enter ->
                                    ( mdl, mdl.keyMod.ctrl |> BX.ifElse (Text mdl.input |> request) Cmd.none )

                                -- TODO get selectionStart of textarea
                                Tab ->
                                    ( mdl.isInput |> BX.ifElse { mdl | input = mdl.input ++ "    " } mdl, Cmd.none )

                                ArrowDown ->
                                    ( mdl.keyMod.ctrl |> BX.ifElse { mdl | isInputFS = True } mdl, Cmd.none )

                                ArrowUp ->
                                    ( mdl.keyMod.ctrl |> BX.ifElse { mdl | isInputFS = False } mdl, Cmd.none )

                                Escape ->
                                    ( mdl, Dom.blur idInput |> Task.attempt (\_ -> NoOp) )

                        AnyKey ->
                            ( mdl, Cmd.none )

                KeyUp key ->
                    case key of
                        Char _ ->
                            ( mdl, Cmd.none )

                        NonChar nc ->
                            case nc of
                                Modifier m ->
                                    ( { mdl | keyMod = mdl.keyMod |> setKeyMod m False }, Cmd.none )

                                _ ->
                                    ( mdl, Cmd.none )

                        AnyKey ->
                            ( mdl, Cmd.none )

                Select tid ->
                    ( { mdl | selected = mdl.selected |> (\l -> List.member tid l |> BX.ifElse (LX.remove tid l) (tid :: l)) }, Cmd.none )

        FromS fromS ->
            case fromS of
                LoggedOut (Ok _) ->
                    ( mdl, U.cmd Goto P.LP )

                Homed (Ok ( _, res )) ->
                    ( { mdl
                        | items = res.items
                        , msg =
                            res.items
                                |> List.isEmpty
                                |> (&&) (res.option |> MX.isNothing)
                                |> BX.ifElse "Nothing to execute, working tree clean."
                                    ([ res.option |> MX.unwrap False ((==) "archives") |> BX.ifElse "Last" ""
                                     , U.len res.items
                                     , res.option |> Maybe.withDefault "items"
                                     , "here."
                                     ]
                                        |> String.join " "
                                    )
                        , timescale = mdl.user.timescale
                        , view =
                            [ "leaves"
                            , "roots"
                            , "archives"
                            ]
                                |> List.map (\s -> res.option == Just s)
                                |> U.overwrite Home_ [ Leaves, Roots, Archives ]
                        , isCurrent = True
                      }
                    , Cmd.none
                    )

                Texted (Ok ( _, res )) ->
                    case res of
                        ResTextC (ResHelp s) ->
                            ( { mdl | input = s }, Cmd.none )

                        ResTextC (ResUser (ResInfo_ r)) ->
                            ( { mdl
                                | msg =
                                    [ "Since " ++ U.clock mdl.user.zone r.since
                                    , "Executed " ++ U.int r.executed
                                    , r.tz
                                    ]
                                        |> String.join ", "
                              }
                            , Cmd.none
                            )

                        ResTextC (ResUser (ResModify m)) ->
                            ( case m of
                                Email s ->
                                    { mdl | msg = "User email modified : " ++ s }

                                Password _ ->
                                    { mdl | msg = "User password modified." }

                                Name s ->
                                    { mdl
                                        | msg = "User name modified : " ++ s
                                        , user =
                                            let
                                                user =
                                                    mdl.user
                                            in
                                            { user | name = s }
                                    }

                                Timescale s ->
                                    { mdl
                                        | msg = "User timescale modified : " ++ s
                                        , timescale = U.timescale s
                                    }
                            , Cmd.none
                            )

                        ResTextC (ResSearch_ r) ->
                            ( { mdl
                                | items = r.items
                                , msg = [ U.len r.items, "search results here." ] |> String.join " "
                                , view = Search
                              }
                            , Cmd.none
                            )

                        ResTextC (ResTutorial_ r) ->
                            ( { mdl
                                | items = r.items
                                , msg = [ U.len r.items, "tutorial items here." ] |> String.join " "
                                , view = Tutorial
                              }
                            , Cmd.none
                            )

                        ResTextT_ r ->
                            ( { mdl
                                | items = r.items
                                , msg =
                                    [ U.int r.created ++ " items created."
                                    , U.int r.updated ++ " items updated."
                                    ]
                                        |> String.join " "
                                , view = Home_
                              }
                            , Cmd.none
                            )

                Cloned (Ok ( _, res )) ->
                    ( { mdl
                        | input = res.text
                        , msg = U.int res.count ++ " items cloned."
                      }
                    , Cmd.none
                    )

                Execed (Ok ( _, res )) ->
                    ( { mdl
                        | items = res.items
                        , msg =
                            [ [ U.int res.count, "items", res.revert |> BX.ifElse "reverted" "executed" ] |> String.join " "
                            , [ "(", U.int res.chain, "chained", ")." ] |> String.join " "
                            ]
                                |> String.join " "
                        , view = Home_
                      }
                    , Cmd.none
                    )

                Focused (Ok ( _, res )) ->
                    ( { mdl
                        | items = res.pred ++ (res.item :: res.succ)
                        , msg =
                            [ "#" ++ U.int res.item.id
                            , "Pred." ++ U.len res.pred
                            , "Succ." ++ U.len res.succ
                            ]
                                |> String.join " "
                        , cursor = List.length res.pred
                        , view = Focus_
                      }
                    , Cmd.none
                    )

                Starred (Ok _) ->
                    ( { mdl | items = mdl.items |> LX.updateAt mdl.cursor (\item -> { item | isStarred = not item.isStarred }) }
                    , Cmd.none
                    )

                LoggedOut (Err e) ->
                    handle mdl e

                Homed (Err e) ->
                    handle mdl e

                Texted (Err e) ->
                    handle mdl e

                Cloned (Err e) ->
                    handle mdl e

                Execed (Err e) ->
                    handle mdl e

                Focused (Err e) ->
                    handle mdl e

                Starred (Err e) ->
                    handle mdl e


handle : Mdl -> U.HttpError -> ( Mdl, Cmd Msg )
handle mdl e =
    ( { mdl | msg = U.strHttpError e }, Cmd.none )


type DU
    = Down
    | Up


follow : DU -> Mdl -> Cmd Msg
follow du mdl =
    let
        h =
            itemHeight |> toFloat

        cursorY =
            mdl.cursor |> toFloat |> (*) h
    in
    Dom.getViewportOf idItems
        |> Task.andThen
            (\info ->
                case du of
                    Down ->
                        if info.viewport.y + info.viewport.height - 3 * h < cursorY then
                            Dom.setViewportOf idItems 0 (cursorY - (info.viewport.height / 2) + 2 * h)

                        else
                            Dom.blur ""

                    Up ->
                        if cursorY < info.viewport.y + h then
                            Dom.setViewportOf idItems 0 (cursorY - (info.viewport.height / 2))

                        else
                            Dom.blur ""
            )
        |> Task.attempt (\_ -> NoOp)


forTheItem : Mdl -> (Item -> Cmd msg) -> Cmd msg
forTheItem mdl f =
    mdl.items |> LX.getAt mdl.cursor |> MX.unwrap Cmd.none f


setKeyMod : Modifier -> Bool -> KeyMod -> KeyMod
setKeyMod m b mod =
    case m of
        Control ->
            { mod | ctrl = b }

        Shift ->
            { mod | shift = b }



-- VIEW


idInput : String
idInput =
    "app__input"


idItems : String
idItems =
    "app__items"


itemHeight : Int
itemHeight =
    40


view : Mdl -> Html Msg
view mdl =
    div [ class "app" ]
        [ div [ class "univ__header" ]
            [ div [ class "app__logo" ] []
            , div [ class "app__inputs" ]
                [ textarea
                    [ id idInput
                    , value mdl.input
                    , onInput Input
                    , onFocus InputFocus
                    , onBlur InputBlur
                    , placeholder Placeholder.placeholder
                    , spellcheck True
                    , classList [ ( "app__input--fullscreen", mdl.isInputFS ) ]
                    ]
                    []
                ]
            , div [ class "app__submits" ]
                [ div [ class "app__btn", class "app__btn--submit", Request (Text mdl.input) |> onClick ] [] ]
            , div [ class "app__user", Request Logout |> onClick ] [ text mdl.user.name ]
            ]
        , div [ class "univ__body" ]
            [ div [ class "app__sidebar" ]
                [ div [ class "app__icons" ]
                    [ div [ class "app__icon", class "app__icon--timescale" ] []
                    , div [ class "app__icon", class "app__icon--timeshift" ] []
                    , div [ class "app__icon", class "app__icon--updown" ] []
                    , div [ class "app__icon", class "app__icon--select" ] []
                    , div [ class "app__icon", class "app__icon--star" ] []
                    , div [ class "app__icon", class "app__icon--focus" ] []
                    , div [ class "app__icon", class "app__icon--url" ] []
                    ]
                ]
            , div [ class "app__main" ]
                [ div [ class "app__nav" ]
                    [ div [ class "app__btns", class "app__btns--edit" ]
                        [ div [ class "app__btn", class "app__btn--invert", KeyDown (Char 'i') |> onClick ] []
                        , div [ class "app__btn", class "app__btn--exec", KeyDown (Char 'e') |> onClick ] []
                        , div [ class "app__btn", class "app__btn--clone", KeyDown (Char 'c') |> onClick ] []
                        ]
                    , div [ class "app__msg" ] [ text mdl.msg ]
                    , div [ class "app__btns", class "app__btns--view" ]
                        [ div [ class "app__btn", class "app__btn--archives", KeyDown (Char 'a') |> onClick ] []
                        , div [ class "app__btn", class "app__btn--roots", KeyDown (Char 'r') |> onClick ] []
                        , div [ class "app__btn", class "app__btn--leaves", KeyDown (Char 'l') |> onClick ] []
                        , div
                            [ class "app__btn"
                            , class "app__btn--home"
                            , KeyDown (Char 'h') |> onClick
                            , classList [ ( "app__btn--home:hover", mdl.view == Home_ ) ] -- TODO :hover works?
                            ]
                            []
                        ]
                    , div [ class "app__scroll", class "app__scroll--nav" ] []
                    ]
                , div [ class "app__table" ]
                    [ div [ class "app__table-header" ]
                        [ div [ class "item__cursor", class "item__cursor--header" ] []
                        , div [ class "item__select", class "item__select--header" ] [ U.len1 mdl.selected |> text ]
                        , div [ class "item__star" ] []
                        , div [ class "item__title" ] []
                        , div [ class "item__startable" ] [ U.strTimescale mdl.timescale |> text ]

                        -- TODO U.class
                        , div [ class "bar" ] [ "As of " ++ U.clock mdl.user.zone mdl.asOf |> text ]
                        , div [ class "deadline" ] [ U.fmtTS mdl.timescale |> text ]
                        , div [ class "priority" ] []
                        , div [ class "weight" ] []
                        , div [ class "assign" ] []
                        , div [ class "scroll" ] []
                        ]
                    , U.enumerate mdl.items |> List.map (viewItem mdl) |> div [ id idItems ]
                    ]
                ]
            , div [ class "app__sidebar" ] []
            ]
        , div [ class "univ__footer" ] []
        ]
        |> Html.map FromU


viewItem : Mdl -> ( Index, Item ) -> Html FromU
viewItem mdl ( idx, item ) =
    let
        isSelected =
            List.member item.id mdl.selected
    in
    div
        [ style "height" (U.int itemHeight ++ "px")
        , classList
            [ ( "item", True )
            , ( "item__status--cursored", idx == mdl.cursor )
            , ( "item__status--selected", isSelected )
            , ( "item__status--overdue", item |> isOverdue mdl )
            , ( "item__status--high-priority", 0 < (item.priority |> Maybe.withDefault 0) )
            ]
        ]
        [ div [ class "item__cursor" ] []
        , div [ class "item__select", Select item.id |> onClick ] [ isSelected |> BX.ifElse "+" "-" |> text ]
        , div [ class "item__star", Request (Star item.id) |> onClick ] [ item.isStarred |> BX.ifElse "★" "☆" |> text ]
        , div [ class "item__title" ] [ item.title |> text |> (\t -> item.link |> MX.unwrap t (\l -> a [ href l, target "_blank" ] [ t ])) ]
        , div [ class "item__startable" ] [ item.startable |> MX.unwrap "-" (U.fmtDT mdl.timescale mdl.user.zone) |> text ]
        , div [ class "item__bar", Request (Focus item.id) |> onClick ] [ item |> dotString mdl |> text ]
        , div [ class "item__deadline" ] [ item.deadline |> MX.unwrap "-" (U.fmtDT mdl.timescale mdl.user.zone) |> text ]
        , div [ class "item__priority" ] [ item.isArchived |> BX.ifElse "X" (item.priority |> MX.unwrap "-" String.fromFloat) |> text ]
        , div [ class "item__weight" ] [ item.weight |> MX.unwrap "-" String.fromFloat |> text ]
        , div [ class "item__assign" ] [ item.assign == mdl.user.name |> BX.ifElse "me" item.assign |> text ]
        ]


isOverdue : Mdl -> Item -> Bool
isOverdue mdl item =
    let
        isOverDeadline =
            item.deadline |> MX.unwrap False (\d -> mdl.now |> U.lt d)
    in
    not item.isArchived && isOverDeadline


timeshift : Mdl -> Int -> Posix -> Posix
timeshift mdl i =
    TX.add mdl.timescale.interval (i * mdl.timescale.multiple) mdl.user.zone


dotString : Mdl -> Item -> String
dotString mdl item =
    let
        inc =
            timeshift mdl 1
    in
    List.range 0 51
        |> List.map
            (\i ->
                let
                    l =
                        U.apply i inc mdl.asOf

                    r =
                        inc l
                in
                dot (Dotter l r) item
            )
        |> String.fromList


type alias Dotter =
    { l : Posix
    , r : Posix
    }


dot : Dotter -> Item -> Char
dot dotter item =
    let
        has =
            MX.unwrap False (\t -> (dotter.l |> U.lt t) && (t |> U.lt dotter.r))

        hasDeadline =
            has item.deadline

        hasStartable =
            has item.startable

        hasStripe =
            item.stripes
                |> List.any
                    (\stripe ->
                        ((stripe.l |> U.lt dotter.l) && (dotter.l |> U.lt stripe.r))
                            || ((dotter.l |> U.lt stripe.l) && (stripe.l |> U.lt dotter.r))
                    )
    in
    U.overwrite '.' [ '#', '[', ']' ] [ hasStripe, hasStartable, hasDeadline ]



-- SUBSCRIPTIONS


subscriptions : Mdl -> Sub Msg
subscriptions mdl =
    Sub.batch
        [ Time.every 1000 Tick
        , decKey |> Decode.map (FromU << KeyDown) |> Events.onKeyDown
        , decKey |> Decode.map (FromU << KeyUp) |> Events.onKeyUp
        ]


decKey : Decoder Key
decKey =
    Decode.field "key" Decode.string
        |> Decode.map
            (\s ->
                case String.uncons s of
                    Just ( c, "" ) ->
                        Char c

                    _ ->
                        case s of
                            "Control" ->
                                Modifier Control |> NonChar

                            "Shift" ->
                                Modifier Shift |> NonChar

                            "Enter" ->
                                NonChar Enter

                            "Tab" ->
                                NonChar Tab

                            "ArrowDown" ->
                                NonChar ArrowDown

                            "ArrowUp" ->
                                NonChar ArrowUp

                            "Escape" ->
                                NonChar Escape

                            _ ->
                                AnyKey
            )


type Key
    = Char Char
    | NonChar NonChar
    | AnyKey


type NonChar
    = Modifier Modifier
    | Enter
    | Tab
    | ArrowDown
    | ArrowUp
    | Escape


type Modifier
    = Control
    | Shift



-- INTERFACE


type Req
    = Logout
    | Home { option : Maybe String }
    | Text String
    | Clone (List Tid)
    | Exec { tids : List Tid, revert : Bool }
    | Focus Tid
    | Star Tid


request : Req -> Cmd Msg
request req =
    case req of
        Logout ->
            U.delete_ EP.Auth (FromS << LoggedOut)

        Home { option } ->
            let
                query =
                    case option of
                        Just s ->
                            [ Url.Builder.string "option" s ]

                        _ ->
                            []
            in
            U.get (EP.Tasks |> EP.App_) query (FromS << Homed) decHome

        Text _ ->
            U.post (EP.Tasks |> EP.App_) (enc req) (FromS << Texted) decText

        Clone _ ->
            U.put (EP.Tasks |> EP.App_) (enc req) (FromS << Cloned) decClone

        Exec _ ->
            U.delete (EP.Tasks |> EP.App_) (enc req) (FromS << Execed) decExec

        Focus tid ->
            U.get (EP.Task tid |> EP.App_) [] (FromS << Focused) decFocus

        Star tid ->
            U.put_ (EP.Task tid |> EP.App_) (FromS << Starred)


enc : Req -> Encode.Value
enc req =
    case req of
        Text text ->
            Encode.object
                [ ( "text", Encode.string text ) ]

        Clone tids ->
            Encode.object
                [ ( "tasks", Encode.list Encode.int tids ) ]

        Exec { tids, revert } ->
            Encode.object
                [ ( "tasks", Encode.list Encode.int tids )
                , ( "revert", Encode.bool revert )
                ]

        _ ->
            Encode.null



-- request home


type alias ResHome =
    { items : List Item
    , option : Maybe String
    }


decHome : Decoder ResHome
decHome =
    Decode.succeed ResHome
        |> required "tasks" (list decItem)
        |> requiredAt [ "query", "option" ] (nullable string)



-- request text


type ResText
    = ResTextC ResTextC
    | ResTextT_ ResTextT


type ResTextC
    = ResHelp String
    | ResUser ResUser
    | ResSearch_ ResSearch
    | ResTutorial_ ResTutorial


type ResUser
    = ResInfo_ ResInfo
    | ResModify ResModify


type alias ResInfo =
    { since : Posix
    , executed : Int
    , tz : String
    }


type ResModify
    = Email String
    | Password ()
    | Name String
    | Timescale String


type alias ResSearch =
    { items : List Item
    }


type alias ResTutorial =
    { items : List Item
    }


type alias ResTextT =
    { items : List Item
    , created : Int
    , updated : Int
    }


decText : Decoder ResText
decText =
    oneOf
        [ Decode.succeed ResTextC
            |> required "Command"
                (oneOf
                    [ Decode.succeed ResHelp
                        |> required "Help" string
                    , Decode.succeed ResUser
                        |> required "User"
                            (oneOf
                                [ Decode.succeed ResInfo
                                    |> requiredAt [ "Info", "since" ] datetime
                                    |> requiredAt [ "Info", "executed" ] int
                                    |> requiredAt [ "Info", "tz" ] string
                                    |> Decode.map ResInfo_
                                , Decode.succeed ResModify
                                    |> required "Modify"
                                        (oneOf
                                            [ Decode.succeed Email
                                                |> required "Email" string
                                            , Decode.succeed Password
                                                |> required "Password" (null ())
                                            , Decode.succeed Name
                                                |> required "Name" string
                                            , Decode.succeed Timescale
                                                |> required "Timescale" string
                                            ]
                                        )
                                ]
                            )
                    , Decode.succeed ResSearch
                        |> requiredAt [ "Search", "tasks" ] (list decItem)
                        |> Decode.map ResSearch_
                    , Decode.succeed ResTutorial
                        |> requiredAt [ "Tutorial", "tasks" ] (list decItem)
                        |> Decode.map ResTutorial_
                    ]
                )
        , Decode.succeed ResTextT
            |> requiredAt [ "Tasks", "tasks" ] (list decItem)
            |> requiredAt [ "Tasks", "info", "created" ] int
            |> requiredAt [ "Tasks", "info", "updated" ] int
            |> Decode.map ResTextT_
        ]



-- request clone


type alias ResClone =
    { text : String
    , count : Int
    }


decClone : Decoder ResClone
decClone =
    Decode.succeed ResClone
        |> required "text" string
        |> requiredAt [ "info", "count" ] int



-- request exec


type alias ResExec =
    { items : List Item
    , count : Int
    , chain : Int
    , revert : Bool
    }


decExec : Decoder ResExec
decExec =
    Decode.succeed ResExec
        |> required "tasks" (list decItem)
        |> requiredAt [ "info", "count" ] int
        |> requiredAt [ "info", "chain" ] int
        |> requiredAt [ "info", "revert" ] bool



-- request focus


type alias ResFocus =
    { item : Item
    , pred : List Item
    , succ : List Item
    }


decFocus : Decoder ResFocus
decFocus =
    Decode.succeed ResFocus
        |> required "task" decItem
        |> required "pred" (list decItem)
        |> required "succ" (list decItem)



-- request


type alias Item =
    { id : Int
    , title : String
    , assign : String
    , isArchived : Bool
    , isStarred : Bool
    , startable : Maybe Posix
    , deadline : Maybe Posix
    , priority : Maybe Float
    , weight : Maybe Float
    , link : Maybe String
    , stripes : List Stripe
    }


type alias Stripe =
    { l : Posix
    , r : Posix
    }


decItem : Decoder Item
decItem =
    Decode.succeed Item
        |> required "id" int
        |> required "title" string
        |> required "assign" string
        |> required "is_archived" bool
        |> required "is_starred" bool
        |> required "startable" (nullable datetime)
        |> required "deadline" (nullable datetime)
        |> required "priority" (nullable float)
        |> required "weight" (nullable float)
        |> required "link" (nullable string)
        |> required "stripes" (list decStripe)


decStripe : Decoder Stripe
decStripe =
    Decode.succeed Stripe
        |> required "l" datetime
        |> required "r" datetime
