# works-sprig

__Under development:__

- [ ] api
- [ ] web/PC
- [ ] web/SP

[demo]: --TODO
[docker]: https://docs.docker.com/get-docker/
[how to email]: https://github.com/satu-n/study-actix-web-simple-auth-server#using-sparkpost-to-send-registration-email
[movie]: --TODO
[tips]: https://github.com/satu-n/tips

## What's this

Sprig the Task Breaker ⚡

[movie][movie]

### Demo

[demo][demo]

### Feature

* 
* 
* 

## How to use

### Input syntax

See input area [placeholder]().

### Shortcuts

#### Input

* __/__ focus input area
* __Ctrl ↓__ maximize input area
* __Ctrl Enter__ submit
* __Ctrl ↑__ minimize input area
* __Esc__ blur input area

#### Cursor

* <img src="web/images/cmd_jk_normal.png" width="20px">
  __JK__ down & up cursor
* <img src="web/images/cmd_x_normal.png" width="20px">
  __X__ select task at cursor
* <img src="web/images/cmd_s_normal.png" width="20px">
  __S__ Star task at cursor
* <img src="web/images/cmd_f_normal.png" width="20px">
  __F__ Focus task at cursor: view directly related tasks
* <img src="web/images/cmd_u_normal.png" width="20px">
  __U__ open URL link of task at cursor
    <!-- - for Slack permalinks, go to the native app message -->

#### Selection

* <img src="web/images/cmd_i_normal.png" width="20px">
  __I__ Invert selection
* <img src="web/images/cmd_e_normal.png" width="20px">
  __E__ Execute and archive selected tasks
* <img src="web/images/cmd_e_normal.png" width="20px">
  __Shift E__ revert selected tasks
* <img src="web/images/cmd_c_normal.png" width="20px">
  __C__ Clone selected tasks to input area
* <img src="web/images/cmd_p_normal.png" width="20px">
  __P__ show critical Path between 2 selected tasks 

#### View

* <img src="web/images/cmd_5_normal.png" width="20px">
  __1-5__ time scale
* <img src="web/images/cmd_wo_normal.png" width="20px">
  __WO__ time shift
* <img src="web/images/cmd_a_normal.png" width="20px">
  __A__ Archives
* <img src="web/images/cmd_r_normal.png" width="20px">
  __R__ Roots, no successor
* <img src="web/images/cmd_l_normal.png" width="20px">
  __L__ Leaves, no predecessor
* <img src="web/images/cmd_h_normal.png" width="20px">
  __H__ Home

### Logout

Click username.

## How to run locally

Prerequisites:

* [Docker & Docker Compose][docker]
* git
* bash

Enter the command as follows to access http://localhost:8080

```bash
```

<!-- ```bash
APP_NAME='my_sprig' &&
git clone https://github.com/satu-n/works-sprig.git $APP_NAME &&
cd $APP_NAME &&
bash init.sh -p $APP_NAME \
'***new!database!password***' &&
unset APP_NAME &&
docker-compose up -d &&
docker-compose logs -f
``` -->

Configure 2 '`single quoted params`'.

`-p` option means Personal use.

`APP_NAME` is now registered as the default user:

* Email: `APP_NAME`
* Password: `APP_NAME`

```bash
docker-compose down
```

to exit the app.
Your data will be retained.

```bash
docker-compose up -d
```

to resume the app.

## Thank you for reading!

See also [my dev tips][tips] if you like.
