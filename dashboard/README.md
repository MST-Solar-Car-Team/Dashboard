# Dashboard Taskboard
This is a list of things that need to be done for the solar car dashboard

### Overview: 
The dashboard will consist of a raspberry pi, attached to a display(dimensions are not decided yet). Data will be received over a CAN port and will be displayed on the dash as well as being transmitted over a cellular antenna. This allow for the collection of telemetry remotely. (Note: The dashboard will only be responsible for sending telemetry data. See [telemetry](https://github.com/connorWinningUM/SolarCarS-T/tree/main/telementry) for more details on how this data will be  processed. 

### Tasks: (More to be added, in order of priority)
- [ ] Get details about screen dimensions from composites team
- [ ]  Inquire about backup camera
- [ ] Make an OS configuration file for the raspberry pi (Nix is the best candidate for OS at the moment) (More to be added)
	- [ ] ~~Python and dependency install~~ (Now using rust, dependency's will be baked into executable, GTK4 and Cairo may still be reqired)
	- [ ] Systemd service for automatic run on 
boot
	- [ ] Wayland?
	- [ ] Solar car logo on boot?
- [ ] Setup display's code, likely in python (~~GTK has been identified as a possible framework~~) (GTK has been chosen)
	- [x] ~~Speedometer widget~~ (done)
	- [ ] Hud lights (for warnings)
	- [ ] Turn signal indicators
	- [ ] Battery percentage and discharge rate (potentially eta)

- [ ] Collection of CAN port data