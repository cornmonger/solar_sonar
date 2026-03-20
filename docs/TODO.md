# TODO: solar_sonar

Things on my radar that may or may not be accomplished in a timely manner.

## Shared API on the local system
Allow multiple instances to run while using only one as the actual data-source.
Useful for efficiently monitoring different data views on separate terminal
tabs or windows. Opens the path to other features.

1. The first instance to run creates a PID file and a data pipe file.
2. Other local instances read data from the pipe file rather than performing
   their own queries.
3. If the main instance is closed, the next running instance will take its place
as the data-source for others.
4. Only the data-source instance will sound audible pings.


## Android app (solar_sonar_mobile)
- Native android with `lib_solar_sonar` integrated via FFI
- Desktop companion paired to an instance of solar_sonar running elsewhere.
- Configuration maintained on the desktop instance. 
- Zero-configuration of the app using Bluetooth pairing.
- Optionally allows user to switch to WiFi after configuration.
- UX: Simple entry lists. Audible alarms as configured on the instance.
- In-app configuration is kept simple:
  - Organization of views
  - Mute or disable things already configured by the instance
- Distrubuted as either a manual `.apk` install or via F-Droid 


## 2D map (solar_sonar_map)
- Iced framework or COSMIC (Iced) framework
- Displays a 2D map as provided by the latest versions of the SDE.
- Pulls intel from a running instance and highlights systems on the map.


## ESI support
- Allow aggregate monitoring of various ESI paths for all characters configured


## Helper commands
- Configure a chat channel (typically an alt channel) for use
- Anything prefixed with `.` gets parsed as a command
- Allow third-party FFI extensions via Stabby crate
- Extensions respond to commands with output:
  - stdout: string or stderr: string
  - string ping text
- Application prints stdout or stderr and sounds a ping.
- Extension configuration:
  - Can provide ping text if allowed by configuration.
    - Default simply sounds for: "Ok" or "Error"
  - Can work with system clipboard if allowed by configuration.
- Primarily useful for (internal):
  - Change application parameters: eg, '.mode attack' 
- Also possibly useful for (external):
  - Inventory appraisal: eg, copy items -> '.appraise' -> URL created for Janice
  - Calculators:
    - Basic: '.calc (32 * (48.0 / -2)) + 5' -> outputs result
    - Courier calculation, based on clipboard and route
  - Soundboard: eg, '.say never gonna give you up' and '.play windows_startup'
  - Ask AI questions: eg, '.ai ECM type for Basilisk?'
  - Ping fleets: eg, '.fleetup' -> pings to slack with information parsed
    from fleet log / MOTD


## Traveling (routes) 
- Differentiate between watching a system and watching a route.
- Monitor each system along the route within N jumps (default: 1 or 2).


## Prompt to alter monitoring
- Ratatui framework, possibly
- The `[spacebar]` key opens a prompt that allows the user to interact.
- Options:
  - TUI Menus
  - TUI terminal
  - or both


## Status bar

Summarize what is currently being monitored.
