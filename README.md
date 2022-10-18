# Maplines
Tool to analyze and visualize GPS data from recorded sport activities.

## Future Features
- [x] add gpx to geojson conversion
- [x] draw line on map
- [x] add user settings (e.g. default smart watches import path, ...)
- [x] import fit files
- [x] load activities from garmin
- [x] on import check if track already present
- [ ] add detailed analysis of tracks (profile & speed calculation, mountain category detection, rounds)
- [x] add activity types
- [x] refactor importing gps files (import by copying and analyzing original files from within program)
- [x] display multiple gps lines (select in table)
- [ ] group gps lines
- [x] add start, stop and pause labels on map
- [x] add deleting of tracks
- [ ] add archiving of tracks
- [x] add editing of tracks
- [ ] add error dialog box
- [ ] multiple pause detection iterations (different radii)
- [ ] use fit field "enhanced speed"
- [x] join tracks
- [ ] add images and comments to track / route
- [ ] map select desired layers to display
- [ ] add custom tags (ride with bike xy, training, trip, ...)
- [ ] add group tags (italy summer 2022, to amsterdam 2017, training for marathon 2024, swiss loppet, ...)
- [ ] add weather tags (sunny, cloudy, rainy, stormy, ...)
- [ ] add infobox to ui elements and attributes (e.g. explain pause / elevation / distance calculation, ...)
- [x] add label on map for current position in profile
- [x] add satellite map option

## More Distant Future Features
- [ ] add track grouping (e.g. group all tracks belonging to a completed tour or tracks with the same route. maybe with tags?)
- [ ] add exporting library feature (compress files)
- [ ] add route calculation
- [ ] add "static routes" and assign recorded tracks to them. (Maybe combine with general route planning...?)
- [ ] multisport activities
- [ ] display heat map
- [ ] distinction of recorded tracks and planned tracks
- [ ] load activities from strava
- [ ] load activities from komoot
- [ ] check for useful fit fields of other devices


## Future Design Improvements
- [ ] map more optimized for biking, hiking, skiing (Less prominent Highways, more focus on cycling ways...)
- [x] improve table design
- [ ] maplines logo
- [ ] color effect on imported tracks
- [x] row selection color
- [ ] import / analysis loading bar

## Changes to Consider (No immediate priority and potentially not useful)
- [ ] reduce number of gps points per line

## Issues
- [ ] When connected to internet but reception is very low, maps can't display gps lines
- [ ] Map does not appear when reconnected to internet
- [ ] When changes to a row are saved, the updated row is not visually selected anymore
- [ ] very scattered elevation data still leads to inaccurate calculations

- [ ] improve pause detection (pauses between only 2 points can be missed)

- [ ] fix elevation not until end of track
- [ ] fix unrealistic elevation correction

## Sources
Geotiff: https://srtm.csi.cgiar.org/wp-content/uploads/files/srtm_5x5/TIFF/