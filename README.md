# works-sprig

__Under development:__

- [ ] Keyboard-oriented mode
- [ ] Touch-oriented mode

<!-- EXTERNAL LINK -->
[demo]: --TODO
[docker]: https://docs.docker.com/get-docker/
[how to email]: https://github.com/satu-n/study-actix-web-simple-auth-server#using-sparkpost-to-send-registration-email
[movie]: --TODO
[tips]: https://github.com/satu-n/tips

<!-- INTERNAL LINK -->
[placeholder]: web/src/Page/App/Placeholder.elm

## What's this

Sprig the Task Breaker ⚡

[movie][movie]

### Demo

[demo][demo]

### Feature

* 
* 
* 

<!-- ### Zen of Sprig

* __Focus on the top task.__
* __Break it down into processable units.__
* __List up, and enter as is.__ -->

## How to use

### Input syntax

See input area [placeholder][placeholder].

### Shortcuts
<!-- TODO timescale 1-9 -->

| Icon | Shortcut | Operation |
| :---: | ---: |---|
|  |  | __INPUT__ |
|  | `/` | focus input area |
|  | `Ctrl` + `↓` | maximize input area |
|  | `Ctrl` + `Enter` | submit |
|  | `Ctrl` + `↑` | minimize input area |
|  | `Esc` | blur input area |
|  |  | __NAVIGATE__ |
| <img src="web/images/cmd_jk_normal.png" width="24px" align="center"> | `J` `K` | down & up cursor |
| <img src="web/images/cmd_x_normal.png" width="24px" align="center"> | `X` | select item at cursor |
| <img src="web/images/cmd_u_normal.png" width="24px" align="center"> | `U` | open URL link of item at cursor |
| <img src="web/images/cmd_i_normal.png" width="24px" align="center"> | `I` | Invert selection |
|  |  | __EDIT__ |
| <img src="web/images/cmd_s_normal.png" width="24px" align="center"> | `S` | Star item at cursor |
| <img src="web/images/cmd_e_normal.png" width="24px" align="center"> | `E` | Execute selected items to archives |
| <img src="web/images/cmd_e_normal.png" width="24px" align="center"> | `Shift` + `E` | revert selected items to home |
| <img src="web/images/cmd_c_normal.png" width="24px" align="center"> | `C` | Clone selected items to input area |
|  |  | __VIEW__ |
| <img src="web/images/cmd_5_normal.png" width="24px" align="center"> | `1` .. `5` | time scale |
| <img src="web/images/cmd_wo_normal.png" width="24px" align="center"> | `W` `O` | time shift |
| <img src="web/images/cmd_f_normal.png" width="24px" align="center"> | `F` | Focus item at cursor: view directly related items |
| <img src="web/images/cmd_a_normal.png" width="24px" align="center"> | `A` | Archives |
| <img src="web/images/cmd_r_normal.png" width="24px" align="center"> | `R` | Roots, no successor |
| <img src="web/images/cmd_l_normal.png" width="24px" align="center"> | `L` | Leaves, no predecessor |
| <img src="web/images/cmd_h_normal.png" width="24px" align="center"> | `H` | Home |

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
