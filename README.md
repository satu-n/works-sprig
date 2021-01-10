# works-sprig

__Under development:__

- [ ] api
- [ ] web/PC
- [ ] web/SP

<!-- EXTERNAL LINK -->
[demo]: --TODO
[docker]: https://docs.docker.com/get-docker/
[how to email]: https://github.com/satu-n/study-actix-web-simple-auth-server#using-sparkpost-to-send-registration-email
[movie]: --TODO
[tips]: https://github.com/satu-n/tips

<!-- INTERNAL LINK -->
[placeholder]: web/_init/src/Page/App/placeholder.txt

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

See input area [placeholder][placeholder].

### Shortcuts


| Icon | Shortcut | Operation |
| :---: | ---: |---|
|  |  | __INPUT__ |
|  | `/` | focus input area |
|  | `Ctrl` + `↓` | maximize input area |
|  | `Ctrl` + `Enter` | submit |
|  | `Ctrl` + `↑` | minimize input area |
|  | `Esc` | blur input area |
|  |  | __CURSOR__ |
| <img src="web/images/cmd_jk_normal.png" width="24px" align="center"> | `J` `K` | down & up cursor |
| <img src="web/images/cmd_x_normal.png" width="24px" align="center"> | `X` | select task at cursor |
| <img src="web/images/cmd_s_normal.png" width="24px" align="center"> | `S` | Star task at cursor |
| <img src="web/images/cmd_f_normal.png" width="24px" align="center"> | `F` | Focus task at cursor: view directly related tasks |
| <img src="web/images/cmd_u_normal.png" width="24px" align="center"> | `U` | open URL link of task at cursor |
|  |  | __SELECTION__ |
| <img src="web/images/cmd_i_normal.png" width="24px" align="center"> | `I` | Invert selection |
| <img src="web/images/cmd_e_normal.png" width="24px" align="center"> | `E` | Execute and archive selected tasks |
| <img src="web/images/cmd_e_normal.png" width="24px" align="center"> | `Shift` + `E` | revert selected tasks |
| <img src="web/images/cmd_c_normal.png" width="24px" align="center"> | `C` | Clone selected tasks to input area |
| <img src="web/images/cmd_p_normal.png" width="24px" align="center"> | `P` | show critical Path between 2 selected tasks  |
|  |  | __VIEW__ |
| <img src="web/images/cmd_5_normal.png" width="24px" align="center"> | `1` .. `5` | time scale |
| <img src="web/images/cmd_wo_normal.png" width="24px" align="center"> | `W` `O` | time shift |
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
