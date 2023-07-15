#!/bin/sh

BINFILE_COMPILED="target/aarch64-unknown-linux-gnu/release/tubefeeder"
BINFILE_ALT="tubefeeder-aarch"
DESKTOPFILE="tubefeeder.desktop"
IMAGEFILE="tubefeeder.png"
BINFILE=$BINFILE_COMPILED

# Check all prerequisites
if [ -z "$1" ]; then
    echo "Two arguments have to be given, the username and then the ip"
    exit
fi

if [ -z "$2" ]; then
    echo "Two arguments have to be given, the username and then the ip"
    exit
fi

if ! [ -f "$BINFILE_COMPILED" ]; then
    BINFILE=$BINFILE_ALT
fi

if ! [ -f "$BINFILE" ]; then
    echo "The Pipeline binary does not seem to exist"
    echo "Make sure you alread have the compiled binary in $BINFILE_COMPILED or the downloaded binary as $BINFILE_ALT"
    exit
fi

echo "Installing on $1@$2"
echo "WARNING: This will override your mpv config on your Pinephone"

echo "Press enter to continue"
read unused

echo "Installing"

# Copy all the files
scp $BINFILE $DESKTOPFILE $IMAGEFILE $1@$2:/tmp

# Create config files and move everything to the right place
ssh $1@$2 <<EOF
mkdir -p ~/.local/bin/
mkdir -p ~/.local/share/icons/
mkdir -p ~/.local/share/tubefeeder/
mkdir -p ~/.local/share/applications/
mkdir -p ~/.config/mpv/
mkdir -p ~/.config/mpv/scripts
mkdir -p ~/.config/mpv/scripts-opt/

sed -i "s/{user}/$1/g" /tmp/tubefeeder.desktop

if [ -f "/tmp/tubefeeder-aarch" ]; then
    mv /tmp/tubefeeder-aarch /tmp/tubefeeder
fi

mv /tmp/tubefeeder ~/.local/bin
mv /tmp/tubefeeder.desktop ~/.local/share/applications
mv /tmp/tubefeeder.png ~/.local/share/icons

touch ~/.config/mpv/mpv.conf
echo "ytdl-format=bestvideo[height<=?720]+worstaudio/worst" > ~/.config/mpv/mpv.conf

touch ~/.config/mpv/scripts/touchscreen-seek.lua
echo "require 'mp.options'

local options = {
    step = 5,
}

read_options(options,'touchscreen-seek')

function touch_seek()
    local pos = get_mouse_area()
    local step = options.step

    if pos == 'left' then
        mp.commandv('seek', -1*step)
    elseif pos == 'right' then
        mp.commandv('seek', step)
    else
        toggle_pause()
    end
end

function get_mouse_area()
    local mouse_x,mouse_y = mp.get_mouse_pos()
    local winX,winY = mp.get_property('osd-width'), mp.get_property('osd-height')

    if mouse_x < winX/3 then
        return 'left'
    elseif mouse_x < winX/3*2 then
        return 'middle'
    else
        return 'right'
    end
end

function toggle_fullscreen()
    local screen_stat = mp.get_property('fullscreen')

    if screen_stat == 'yes' then
        screen_stat = 'no'
    else
        screen_stat = 'yes'
    end

    mp.set_property('fullscreen', screen_stat)
end

function toggle_pause()
    local pause = mp.get_property_native('pause')
    mp.set_property_native('pause', not pause)
end


mp.add_key_binding(nil, 'touchscreen-seek', touch_seek)
mp.msg.info('Loaded touchscreen-seek Lua flavor')" > ~/.config/mpv/scripts/touchscreen-seek.lua

echo "step = 5" > ~/.config/mpv/scripts-opt/touchscreen-seek.conf

echo "MBTN_LEFT_DBL script-binding touchscreen-seek" > ~/.config/mpv/input.conf

EOF

echo "Successfully installed Pipeline"
echo "Dont forget installing mpv and youtube-dl"
