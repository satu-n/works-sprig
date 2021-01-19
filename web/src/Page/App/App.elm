module Page.App.App exposing (..)

import Bool.Extra as BX
import Browser.Dom as Dom
import Browser.Events as Events
import EndPoint as EP
import Html exposing (..)
import Html.Attributes exposing (alt, classList, href, placeholder, spellcheck, src, target, value)
import Html.Events exposing (onBlur, onClick, onFocus, onInput)
import Json.Decode as Decode exposing (Decoder, bool, float, int, list, null, nullable, oneOf, string)
import Json.Decode.Extra exposing (datetime)
import Json.Decode.Pipeline exposing (required, requiredAt)
import Json.Encode as Encode
import List.Extra as LX
import Maybe.Extra as MX
import Page as P
import Page.App.Placeholder as Placeholder
import String.Extra as SX
import Task
import Time exposing (Posix)
import Time.Extra as TX
import Url.Builder as UB
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
    = None
    | Home_
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
      , view = None
      , now = Time.millisToPosix 0
      , asOf = Time.millisToPosix 0
      , isCurrent = True
      , isInput = False
      , isInputFS = False
      , keyMod = KeyMod False False
      }
      --   TODO intro animation
    , Cmd.none
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
                                            ( mdl, U.idBy "app" "input" |> Dom.focus |> Task.attempt (\_ -> NoOp) )

                                        'w' ->
                                            ( { mdl | asOf = mdl.asOf |> timeshift mdl -1, isCurrent = False }, Cmd.none )

                                        'o' ->
                                            ( { mdl | asOf = mdl.asOf |> timeshift mdl 1, isCurrent = False }, Cmd.none )

                                        'j' ->
                                            ( { mdl | cursor = mdl.cursor < List.length mdl.items - 1 |> BX.ifElse (mdl.cursor + 1) mdl.cursor }, follow Down mdl )

                                        'k' ->
                                            ( { mdl | cursor = 0 < mdl.cursor |> BX.ifElse (mdl.cursor - 1) mdl.cursor }, follow Up mdl )

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
                                    ( mdl, U.idBy "app" "input" |> Dom.blur |> Task.attempt (\_ -> NoOp) )

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
                                     , res.items |> List.length |> singularize (res.option |> Maybe.withDefault "items")
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
                                |> input0
                            , Cmd.none
                            )

                        ResTextC (ResUser (ResModify m)) ->
                            ( (case m of
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
                              )
                                |> input0
                            , Cmd.none
                            )

                        ResTextC (ResSearch_ r) ->
                            ( { mdl
                                | items = r.items
                                , msg =
                                    [ (r.items |> List.length |> singularize "hits") ++ ":"
                                    , -- TODO actual search condition
                                      "actual search condition"
                                    ]
                                        |> String.join " "
                                , view = Search
                              }
                            , Cmd.none
                            )

                        ResTextC (ResTutorial_ r) ->
                            ( { mdl
                                | items = r.items
                                , msg =
                                    [ r.items |> List.length |> singularize "materials"
                                    , "here."
                                    ]
                                        |> String.join " "
                                , view = Tutorial
                              }
                                |> input0
                            , Cmd.none
                            )

                        ResTextT_ r ->
                            ( { mdl
                                | items = r.items
                                , msg =
                                    [ r.created |> singularize "items"
                                    , "created."
                                    , r.updated |> singularize "items"
                                    , "updated."
                                    ]
                                        |> String.join " "
                                , view = Home_
                              }
                                |> input0
                            , Cmd.none
                            )

                Cloned (Ok ( _, res )) ->
                    ( { mdl
                        | input = res.text
                        , msg =
                            [ res.count |> singularize "items"
                            , "cloned."
                            ]
                                |> String.join " "
                      }
                    , Cmd.none
                    )

                Execed (Ok ( _, res )) ->
                    ( { mdl
                        | items = res.items
                        , msg =
                            [ [ res.count |> singularize "items"
                              , res.revert |> BX.ifElse "reverted" "executed"
                              ]
                                |> String.join " "
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

        theId =
            U.idBy "app" "items"
    in
    Dom.getViewportOf theId
        |> Task.andThen
            (\info ->
                let
                    top =
                        info.viewport.y

                    bottom =
                        top + info.viewport.height

                    setAtCursor =
                        \adjust condition ->
                            condition
                                |> BX.ifElse
                                    (Dom.setViewportOf theId 0 (cursorY - (info.viewport.height / 2) + adjust))
                                    (Dom.blur "")
                in
                case du of
                    Down ->
                        bottom - 3 * h < cursorY |> setAtCursor (2 * h)

                    Up ->
                        cursorY < top + h |> setAtCursor 0
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


input0 : Mdl -> Mdl
input0 mdl =
    { mdl | input = "" }


singularize : String -> Int -> String
singularize plural i =
    [ ( "items", "item" )
    , ( "leaves", "leaf" )
    , ( "roots", "root" )
    , ( "archives", "archive" )
    , ( "hits", "hit" )
    , ( "materials", "material" )
    ]
        |> LX.find (\( p, _ ) -> p == plural)
        |> MX.unwrap plural (\( p, s ) -> SX.pluralize s p i)



-- VIEW


itemHeight : Int
itemHeight =
    40


imgDir : String
imgDir =
    "../images"


view : Mdl -> Html Msg
view mdl =
    let
        block =
            "app"

        idBy =
            \elem -> U.idBy block elem |> Html.Attributes.id

        bem =
            U.bem block

        img_ =
            \alt_ basename -> img [ alt alt_, UB.relative [ imgDir, basename ++ ".png" ] [] |> src ] []

        toCharBtn =
            \cl mod ->
                let
                    char =
                        mod |> U.unconsOr ' '
                in
                button
                    [ bem "btn" []
                    , classList cl
                    , KeyDown (Char char) |> onClick
                    ]
                    [ img_ mod ("cmd_" ++ String.fromChar char) ]

        toEditBtn =
            toCharBtn []

        toViewBtn =
            \mod -> toCharBtn [ ( "on", mod |> asView |> MX.unwrap False (\v -> v == mdl.view) ) ] mod

        item__ =
            \elem -> U.bem "item" elem [ ( "header", True ) ]
    in
    div [ bem "" [] ]
        [ header [ bem "header" [] ]
            [ div [ bem "logos" [] ]
                [ div [ bem "logo" [] ] [ img_ "logo" "logo" ] ]
            , div [ bem "inputs" [] ]
                [ textarea
                    [ idBy "input"
                    , bem "input" [ ( "fullscreen", mdl.isInputFS ) ]
                    , value mdl.input
                    , onInput Input
                    , onFocus InputFocus
                    , onBlur InputBlur
                    , placeholder Placeholder.placeholder
                    , spellcheck True
                    ]
                    []
                ]
            , div [ bem "submits" [] ]
                [ button [ bem "btn" [ ( "submit", True ) ], Request (Text mdl.input) |> onClick ] [ img_ "submit" "sprig" ] ]
            , div [ bem "accounts" [] ]
                [ button [ bem "btn" [ ( "account", True ) ], Request Logout |> onClick ] [ span [] [ text mdl.user.name ] ] ]
            ]
        , div [ bem "body" [] ]
            [ div [ bem "sidebar" [] ]
                [ ul [ bem "icons" [] ]
                    ([ ( "timescale", "1-9" )
                     , ( "timeshift", "wo" )
                     , ( "updown", "jk" )
                     , ( "select", "x" )
                     , ( "star", "s" )
                     , ( "focus", "f" )
                     , ( "url", "u" )
                     ]
                        |> List.map (\( mod, key ) -> li [ bem "icon" [] ] [ img_ mod ("cmd_" ++ key) ])
                    )
                ]
            , main_ [ bem "main" [] ]
                [ nav [ bem "nav" [] ]
                    [ div [ bem "btns" [ ( "edit", True ) ] ]
                        ([ "invert", "exec", "clone" ] |> List.map toEditBtn)
                    , div [ bem "msg" [] ] [ span [] [ text mdl.msg ] ]
                    , div [ bem "btns" [ ( "view", True ) ] ]
                        ([ "archives", "roots", "leaves", "home" ] |> List.map toViewBtn)
                    , div [ bem "scroll" [] ] []
                    ]
                , table [ bem "table" [] ]
                    [ thead [ bem "table-header" [] ]
                        [ th [ item__ "cursor" ] []
                        , th [ item__ "select" ] [ U.len1 mdl.selected |> text ]
                        , th [ item__ "star" ] []
                        , th [ item__ "title" ] []
                        , th [ item__ "startable" ] [ U.strTimescale mdl.timescale |> text ]
                        , th [ item__ "bar" ] [ span [] [ "As of " ++ U.clock mdl.user.zone mdl.asOf |> text ] ]
                        , th [ item__ "deadline" ] [ U.fmtTS mdl.timescale |> text ]
                        , th [ item__ "priority" ] []
                        , th [ item__ "weight" ] []
                        , th [ item__ "assign" ] []
                        , th [ bem "scroll" [] ] []
                        ]
                    , U.enumerate mdl.items
                        |> List.map (viewItem mdl)
                        |> tbody [ idBy "items", bem "items" [] ]
                    ]
                ]
            , div [ bem "sidebar" [ ( "pad-scroll", True ) ] ] []
            ]
        , footer [ bem "footer" [] ] []
        ]
        |> Html.map FromU


asView : String -> Maybe View
asView s =
    [ "home"
    , "leaves"
    , "roots"
    , "archives"
    , "focus"
    , "search"
    , "tutorial"
    ]
        |> List.map ((==) s)
        |> U.overwrite Nothing
            ([ Home_
             , Leaves
             , Roots
             , Archives
             , Focus_
             , Search
             , Tutorial
             ]
                |> List.map Just
            )


viewItem : Mdl -> ( Index, Item ) -> Html FromU
viewItem mdl ( idx, item ) =
    let
        bem =
            U.bem "item"

        isSelected =
            List.member item.id mdl.selected
    in
    tr
        [ Html.Attributes.style "height" (U.int itemHeight ++ "px")
        , bem "" [ ( "selected", isSelected ) ]
        ]
        [ td [ bem "cursor" [ ( "spot", idx == mdl.cursor ) ] ] []
        , td [ bem "select" [], Select item.id |> onClick ] [ isSelected |> BX.ifElse "+" "-" |> text ]
        , td [ bem "star" [], Request (Star item.id) |> onClick ] [ item.isStarred |> BX.ifElse "★" "☆" |> text ]
        , td [ bem "title" [] ] [ span [] [ item.title |> text |> (\t -> item.link |> MX.unwrap t (\l -> a [ href l, target "_blank" ] [ t ])) ] ]
        , td [ bem "startable" [] ] [ item.startable |> MX.unwrap "-" (U.fmtDT mdl.timescale mdl.user.zone) |> text ]
        , td [ bem "bar" [], Request (Focus item.id) |> onClick ] [ item |> dotString mdl |> text ]
        , td
            [ bem "deadline" [ ( "overdue", item |> isOverdue mdl ) ] ]
            [ item.deadline |> MX.unwrap "-" (U.fmtDT mdl.timescale mdl.user.zone) |> text ]
        , td
            [ bem "priority" [ ( "high", 0 < (item.priority |> Maybe.withDefault 0) ) ] ]
            [ item.isArchived |> BX.ifElse "X" (item.priority |> MX.unwrap "-" priority) |> text ]
        , td [ bem "weight" [] ] [ item.weight |> MX.unwrap "-" weight |> text ]
        , td [ bem "assign" [] ] [ span [] [ item.assign == mdl.user.name |> BX.ifElse "me" item.assign |> text ] ]
        ]


priority : Float -> String
priority x =
    [ x < -1000, 1000 < x ] |> U.overwrite (U.signedDecimal 1 x) [ "--", "high" ]


weight : Float -> String
weight x =
    [ 10000 < x ] |> U.overwrite (U.decimal 1 x) [ "heavy" ]


isOverdue : Mdl -> Item -> Bool
isOverdue mdl item =
    let
        isOverDeadline =
            item.deadline |> MX.unwrap False (\d -> d |> U.lt mdl.now)
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
                            [ UB.string "option" s ]

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
