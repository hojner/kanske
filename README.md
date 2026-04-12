# What is Kanske?

## The name

The name "kanske" (IPA: /ˈkanːɧɛ/) is Swedish for "maybe". Which, at the time of
naming, was a rough estimate of the probability that project would reach Prod
status. The word is also pretty similar to Kanshi, the excellent app I'm
rewriting.

## The functionality

Arch Wiki says this about Kanshi: \*"Kanshi allows you to define output profiles
that are automatically enabled and disabled on hotplug. For instance, this can
be used to turn a laptop's internal screen off when docked.

This is a Wayland equivalent for tools like autorandr. kanshi can be used on
Wayland compositors supporting the wlr-output-management protocol."\*

Kanske is meant to be a Rust re-write of Kanshi, for no good reason other than
it being a fun project. You should use Kanshi, it's great!

## The disclaimer

Look, I'm no developer. Neither a Rust developer nor a C developer. This is
probably full of bugs and logic errors. Don't use it unless you feel ok with
that. I take no responsibility for complications to your setup as a result of
using this app.

## The safety

Kanske offers some safety improvements. Apart from Rust's built-in memory safety
guarantees, the app will trigger notify-send if the config file is not owned by
current user or root or if it is group- or world-writable. This is due to the
exec and include directives that would offer an attack vector for executing
shell commands. This is, of course, no guarantee that this vector can't be
exploited but the notification offers some support to the user. This approach is
used in other security critical daemons but, as far as I understand, not in
Kanshi.

There are, at the time of writing, no unsafe blocks in Kanske.

## The service enabling

Kanske is a long-running daemon, so you'll typically want to start it
automatically with your Wayland session.

### Sway

Add to your Sway config (`~/.config/sway/config`):

```
exec kanske
```

Note: logs from `exec` are not easily accessible. If you need to debug, prefer
the systemd approach below.

### systemd (recommended)

Create `~/.config/systemd/user/kanske.service`:

```ini
[Unit]
Description=Kanske dynamic display configuration
Documentation=https://github.com/hojner/kanske
BindsTo=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=/path/to/kanske
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical-session.target
```

`graphical-session.target` is the portable freedesktop standard and works across
compositors. If you prefer, you can substitute your compositor-specific target
(e.g. `sway-session.target` or `hyprland-session.target`).

Then enable and start it:

```sh
systemctl --user daemon-reload
systemctl --user enable --now kanske
```

Logs are available via the journal:

```sh
journalctl --user -u kanske           # all logs
journalctl --user -u kanske -f        # follow live
```

To change the log level, override the environment in the service:

```sh
systemctl --user edit kanske
```

And add:

```ini
[Service]
Environment=RUST_LOG=kanske=debug,kanske_lib=debug
```
