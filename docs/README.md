


# Command summary

  * [`zoom-sync`↴](#zoom-sync)
  * [`zoom-sync set`↴](#zoom-sync-set)
  * [`zoom-sync set time`↴](#zoom-sync-set-time)
  * [`zoom-sync set weather`↴](#zoom-sync-set-weather)
  * [`zoom-sync set system`↴](#zoom-sync-set-system)
  * [`zoom-sync set screen`↴](#zoom-sync-set-screen)
  * [`zoom-sync set image`↴](#zoom-sync-set-image)
  * [`zoom-sync set image clear`↴](#zoom-sync-set-image-clear)
  * [`zoom-sync set gif`↴](#zoom-sync-set-gif)
  * [`zoom-sync set gif clear`↴](#zoom-sync-set-gif-clear)
  * [`zoom-sync set clear`↴](#zoom-sync-set-clear)

## zoom-sync

Cross-platform utility for syncing zoom65v3 screen modules

**Usage**: **`zoom-sync`** (\[**`-S`**=_`ARG`_\] \[**`-W`**=_`ARG`_\] \[**`-R`**=_`ARG`_\] \[**`-f`**\] \[**`--reactive`** | (**`-s`**=_`POSITION`_ | **`--up`** | **`--down`** | **`--switch`**)\] (**`--no-weather`** | \[**`--coords`** _`LAT`_ _`LON`_\] | **`-w`** _`WMO`_ _`CUR`_ _`MIN`_ _`MAX`_) (**`--no-system`** | (\[**`--cpu`**=_`LABEL`_\] | **`-c`**=_`TEMP`_) (\[**`--gpu`**=_`ID`_\] | **`-g`**=_`TEMP`_) \[**`-d`**=_`ARG`_\]) | _`COMMAND ...`_)



**Screen options:**
- **`    --reactive`** &mdash; 
  Enable reactive mode, playing gif when typing and image when resting. Requires root permission for reading keypresses via evdev
- **`-s`**, **`--screen`**=_`POSITION`_ &mdash; 
  Reset and move the screen to a specific position. [cpu|gpu|download|time|weather|meletrix|zoom65|image|gif|battery]
- **`    --up`** &mdash; 
  Move the screen up
- **`    --down`** &mdash; 
  Move the screen down
- **`    --switch`** &mdash; 
  Switch the screen offset



**Weather forecast options:**
- **`    --no-weather`** &mdash; 
  Disable updating weather info completely
### **`--coords`** _`LAT`_ _`LON`_
- **`    --coords`** &mdash; 
  Optional coordinates to use for fetching weather data, skipping ipinfo geolocation api.
- _`LAT`_ &mdash; 
  Latitude
- _`LON`_ &mdash; 
  Longitude


### **`-w`** _`WMO`_ _`CUR`_ _`MIN`_ _`MAX`_
- **`-w`**, **`--weather`** &mdash; 
  Manually provide weather data, skipping open-meteo weather api. All values are unitless.
- _`WMO`_ &mdash; 
  WMO Index
- _`CUR`_ &mdash; 
  Current temperature
- _`MIN`_ &mdash; 
  Minumum temperature
- _`MAX`_ &mdash; 
  Maximum temperature





**System info options:**
- **`    --no-system`** &mdash; 
  Disable updating system info completely
- **`    --cpu`**=_`LABEL`_ &mdash; 
  Sensor label to search for
   
  [default: coretemp Package]
- **`-c`**, **`--cpu-temp`**=_`TEMP`_ &mdash; 
  Manually set CPU temperature
- **`    --gpu`**=_`ID`_ &mdash; 
  GPU device id to fetch temperature data for (nvidia only)
   
  [default: 0]
- **`-g`**, **`--gpu-temp`**=_`TEMP`_ &mdash; 
  Manually set GPU temperature
- **`-d`**, **`--download`**=_`ARG`_ &mdash; 
  Manually set download speed



**Available options:**
- **`-S`**, **`--refresh-system`**=_`ARG`_ &mdash; 
  Interval in seconds to refresh system data
   
  [default: 10s]
- **`-W`**, **`--refresh-weather`**=_`ARG`_ &mdash; 
  Interval in seconds to refresh weather data
   
  [default: 1h]
- **`-R`**, **`--retry`**=_`ARG`_ &mdash; 
  Retry interval for reconnecting to keyboard
   
  [default: 5s]
- **`-f`**, **`--farenheit`** &mdash; 
  Use farenheit for all fetched temperatures. May cause clamping for anything greater than 99F. No effect on any manually provided data.
- **`-h`**, **`--help`** &mdash; 
  Prints help information
- **`-V`**, **`--version`** &mdash; 
  Prints version information



**Available commands:**
- **`set`** &mdash; 
  Set specific options on the keyboard.


## zoom-sync set

Set specific options on the keyboard. Must not be used while zoom-sync is already running.

**Usage**: **`zoom-sync`** **`set`** _`COMMAND ...`_

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information



**Available commands:**
- **`time`** &mdash; 
  Sync time to system clock
- **`weather`** &mdash; 
  Set weather data
- **`system`** &mdash; 
  Set system info
- **`screen`** &mdash; 
  Change current screen
- **`image`** &mdash; 
  Upload static image
- **`gif`** &mdash; 
  Upload animated image (gif/webp/apng)
- **`clear`** &mdash; 
  Clear all media files


## zoom-sync set time

Sync time to system clock

**Usage**: **`zoom-sync`** **`set`** **`time`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set weather

Set weather data

**Usage**: **`zoom-sync`** **`set`** **`weather`** \[**`-f`**\] (**`--no-weather`** | \[**`--coords`** _`LAT`_ _`LON`_\] | **`-w`** _`WMO`_ _`CUR`_ _`MIN`_ _`MAX`_)

**Weather forecast options:**
- **`    --no-weather`** &mdash; 
  Disable updating weather info completely
### **`--coords`** _`LAT`_ _`LON`_
- **`    --coords`** &mdash; 
  Optional coordinates to use for fetching weather data, skipping ipinfo geolocation api.
- _`LAT`_ &mdash; 
  Latitude
- _`LON`_ &mdash; 
  Longitude


### **`-w`** _`WMO`_ _`CUR`_ _`MIN`_ _`MAX`_
- **`-w`**, **`--weather`** &mdash; 
  Manually provide weather data, skipping open-meteo weather api. All values are unitless.
- _`WMO`_ &mdash; 
  WMO Index
- _`CUR`_ &mdash; 
  Current temperature
- _`MIN`_ &mdash; 
  Minumum temperature
- _`MAX`_ &mdash; 
  Maximum temperature





**Available options:**
- **`-f`**, **`--farenheit`** &mdash; 
  Use farenheit for all fetched temperatures. May cause clamping for anything greater than 99F. No effect on any manually provided data.
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set system

Set system info

**Usage**: **`zoom-sync`** **`set`** **`system`** \[**`-f`**\] (\[**`--cpu`**=_`LABEL`_\] | **`-c`**=_`TEMP`_) (\[**`--gpu`**=_`ID`_\] | **`-g`**=_`TEMP`_) \[**`-d`**=_`ARG`_\]

**Available options:**
- **`-f`**, **`--farenheit`** &mdash; 
  Use farenheit for all fetched temperatures. May cause clamping for anything greater than 99F. No effect on any manually provided data.
- **`    --cpu`**=_`LABEL`_ &mdash; 
  Sensor label to search for
   
  [default: coretemp Package]
- **`-c`**, **`--cpu-temp`**=_`TEMP`_ &mdash; 
  Manually set CPU temperature
- **`    --gpu`**=_`ID`_ &mdash; 
  GPU device id to fetch temperature data for (nvidia only)
   
  [default: 0]
- **`-g`**, **`--gpu-temp`**=_`TEMP`_ &mdash; 
  Manually set GPU temperature
- **`-d`**, **`--download`**=_`ARG`_ &mdash; 
  Manually set download speed
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set screen

Change current screen

**Usage**: **`zoom-sync`** **`set`** **`screen`** (**`-s`**=_`POSITION`_ | **`--up`** | **`--down`** | **`--switch`**)

**Screen options:**
- **`-s`**, **`--screen`**=_`POSITION`_ &mdash; 
  Reset and move the screen to a specific position. [cpu|gpu|download|time|weather|meletrix|zoom65|image|gif|battery]
- **`    --up`** &mdash; 
  Move the screen up
- **`    --down`** &mdash; 
  Move the screen down
- **`    --switch`** &mdash; 
  Switch the screen offset



**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set image

Upload static image

**Usage**: **`zoom-sync`** **`set`** **`image`** (\[**`-n`**\] \[**`-b`**=_`ARG`_\] _`PATH`_ | _`COMMAND ...`_)

**Available positional items:**
- _`PATH`_ &mdash; 
  Path to image to re-encode and upload



**Available options:**
- **`-n`**, **`--nearest`** &mdash; 
  Use nearest neighbor interpolation when resizing, otherwise uses gaussian
- **`-b`**, **`--bg`**=_`ARG`_ &mdash; 
  Optional background color for transparent images
   
  [default: #000000]
- **`-h`**, **`--help`** &mdash; 
  Prints help information



**Available commands:**
- **`clear`** &mdash; 
  Delete the content, resetting back to the default.


## zoom-sync set image clear

Delete the content, resetting back to the default.

**Usage**: **`zoom-sync`** **`set`** **`image`** **`clear`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set gif

Upload animated image (gif/webp/apng)

**Usage**: **`zoom-sync`** **`set`** **`gif`** (\[**`-n`**\] \[**`-b`**=_`ARG`_\] _`PATH`_ | _`COMMAND ...`_)

**Available positional items:**
- _`PATH`_ &mdash; 
  Path to image to re-encode and upload



**Available options:**
- **`-n`**, **`--nearest`** &mdash; 
  Use nearest neighbor interpolation when resizing, otherwise uses gaussian
- **`-b`**, **`--bg`**=_`ARG`_ &mdash; 
  Optional background color for transparent images
   
  [default: #000000]
- **`-h`**, **`--help`** &mdash; 
  Prints help information



**Available commands:**
- **`clear`** &mdash; 
  Delete the content, resetting back to the default.


## zoom-sync set gif clear

Delete the content, resetting back to the default.

**Usage**: **`zoom-sync`** **`set`** **`gif`** **`clear`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


## zoom-sync set clear

Clear all media files

**Usage**: **`zoom-sync`** **`set`** **`clear`** 

**Available options:**
- **`-h`**, **`--help`** &mdash; 
  Prints help information


