# nanoleaf-cli
Check the [releases](https://github.com/Smelliott94/nanoleaf-cli/releases) for the latest binary.

A simple and fast CLI for interacting with a set of nanoleaf panels on your home network.

I use this at home mainly to turn my nanoleaf lights on and off at the same time as
my PC by [configuring it as a startup/shutdown script on Windows](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2012-r2-and-2012/dn789190(v=ws.11))
and to quickly switch to a "warm white" light in the evenings without having to
load a UI app or cycle through effects using the controller.

I am very much a Rust novice so feedback is greatly appreciated!

## Setup
**Requires the Nanoleaf device to be connected the same network as your computer**

If you know the IP address of your nanoleaf device already:
```bash
nanoleaf set_ip {your_ip}
# hold the power button on your nanoleaf for 5-7s until the LED flashes in a pattern
nanoleaf pair
```

If you don't, you can find it with the devices MAC address (can be found on the nanoleaf controller) or enough of it to return a unique IP match.
**on Linux / MacOS**
```bash
nanoleaf discover {MAC or MAC substring}
# hold the power button on your nanoleaf for 5-7s until the LED flashes in a pattern
nanoleaf pair
```

**on Windows (powershell)**
```powershell
arp -a | Select-String {your_nanoleaf_mac_address}
# copy your ip from the output, if there's a match (idk powershell lol)
nanoleaf.exe set_ip {your_nanoleaf_ip}
nanoleaf.exe pair
```

The IP and auth token will be stored in a .nanoleaf file in your home directory

### Commands
```bash
# List commands
nanoleaf --help
```

```bash
# Lights on
nanoleaf on
```

```bash
# Lights off
nanoleaf off
```

```bash
# List available effects for your nanoleaf
nanoleaf effect -l
```

```bash
# Activate a nanoleaf effect
nanoleaf effect "Effect name"
```
