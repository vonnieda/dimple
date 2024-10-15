MVP

- I can add MP3 and FLAC tracks by selecting a directory.
	- Adding the same track twice should not duplicate it.
	- Title, artist, album, length tags are stored as metadata for the file and shown in lists below.
- I can see a list of tracks in my library.
	- Including title, artist, album, length.
- I can add tracks to a play queue list.
- I can see a play queue list.
	- Including title, artist, album, length.
- I can start playing the play queue and it will play until completion.
- I can run the GUI both on Mac and Windows.
- I automatically sync library changes between my laptop and desktop using S3.
	- Files and metadata are uploaded to S3.
	- I can "Like" a track on my laptop and see the change reflected on Windows without manually refreshing.
		- The UI reacts to changes in the data store.
	- MP3 files added on laptop are also visible and playable on desktop, and vice-versa. Metadata should be synced immediately and files will be downloaded on demand.
