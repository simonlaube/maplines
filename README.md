# Maplines
Small tool to analyze and visualize GPS data from recorded sport activities.

## Future Features
- [x] add gpx to geojson conversion
- [x] draw line on map
- [ ] add user settings (e.g. default smart watches import path, ...)
- [x] import fit files
- [ ] load activities from garmin / strava...
- [ ] reduce number of gps points per line
- [ ] try to approximate gps lines to streets of OSM
- [ ] add detailed analysis of tracks (profile & speed calculation, mountain category detection, rounds)
- [ ] add activity types
- [x] refactor importing gps files (import by copying and analyzing original files from within program)
- [ ] display multiple gps lines (select in table)
- [ ] group gps lines
- [ ] add route creation
- [ ] display heat map
- [ ] distinction of recorded tracks and planned tracks
- [ ] add start, stop and pause labels on map

## More Distant Future Features
- [ ] add tags
- [ ] add track grouping (e.g. group all tracks belonging to a completed tour or tracks with the same route. maybe with tags?)
- [ ] add deleting & archiving of tracks
- [ ] add editing of tracks and their metadata
- [ ] add exporting library feature (compress files)
- [ ] add route calculation
- [ ] add "static routes" and assign recorded tracks to them. (Maybe combine with general route planning...?)
- [ ] multisport activities


## Future Design Improvements
- [ ] map more optimized for biking, hiking, skiing (Less prominent Highways, more focus on cycling ways...)
- [x] improve table design
- [ ] maplines logo
- [ ] color effect on imported tracks
- [ ] row selection color
- [ ] import / analysis loading bar

## Changes to Consider (No immediate priority)
- [ ] use cbor instead of json / geojson / gpx

## Issues
- [ ] When connected to internet but reception is very low, maps can't display gps lines