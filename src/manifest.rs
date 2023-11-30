use std::{fs::File, io::Write, path::PathBuf};

pub fn create_manifest_file_subs(
    id: &str,
    manifest_subs_path: &PathBuf,
    duration: f32,
) -> std::io::Result<()> {
    // todo change by length
    let chunks: f32 = duration / 10.0;
    let remainder: f32 = duration % 10.0;
    let chunks: usize = chunks.ceil() as usize;

    let mut file_buffer = File::create(manifest_subs_path)?;

    let mut list_of_chunks: Vec<String> = (0..chunks - 1)
        .map(|chunk| format!("#EXTINF: 10.000000,\n{}{}.vtt", id, chunk))
        .collect();

    let remainder_str = {
        if remainder == 0.0 {
            format!("#EXTINF: 10.000000,\n{}{}.vtt", id, chunks)
        } else {
            format!("#EXTINF: {:.6},\n{}{}.vtt", remainder, id, chunks)
        }
    };

    list_of_chunks.push(remainder_str);
    let chunks_str = list_of_chunks.join("\n");

    let str_to_write = format!("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:11\n#EXT-X-MEDIA-SEQUENCE:0\n{}\n#EXT-X-ENDLIST\n", chunks_str);
    write!(file_buffer, "{}", str_to_write)?;

    Ok(())
}

pub fn create_manifest_file(
    id: &str,
    manifest_path: &PathBuf,
    duration: f32,
    init_seg: &str,
) -> std::io::Result<()> {
    let chunks: f32 = duration / 10.0;
    let remainder: f32 = duration % 10.0;
    let chunks: usize = chunks.ceil() as usize;

    let mut file_buffer = File::create(manifest_path)?;

    let mut list_of_chunks: Vec<String> = (0..chunks - 1)
        .map(|chunk| format!("#EXTINF: 10.000000,\n{}_{}.m4s", id, chunk))
        .collect();

    let remainder_str = {
        if remainder == 0.0 {
            format!("#EXTINF: 10.000000,\n{}_{}.m4s", id, chunks - 1)
        } else {
            format!("#EXTINF: {:.6},\n{}_{}.m4s", remainder, id, chunks - 1)
        }
    };

    list_of_chunks.push(remainder_str);
    let chunks_str = list_of_chunks.join("\n");
    let str_to_write = format!("#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:11\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-MAP:URI=\"{}\"\n{}\n#EXT-X-ENDLIST\n", init_seg, chunks_str);
    write!(file_buffer, "{}", str_to_write)?;
    Ok(())
}

pub fn create_master_file(
    manifest_path: &PathBuf,
    manifest_subs_path: &PathBuf,
    playlist_path: &PathBuf,
    no_subs: bool,
) -> std::io::Result<()> {
    let mut file_buffer = File::create(&playlist_path)?;
    let manifest_subs_path = manifest_subs_path.file_name().unwrap().to_str().unwrap();
    let manifest_path = manifest_path.file_name().unwrap().to_str().unwrap();
    // let str_to_write = format!("#EXTM3U\n#EXT-X-VERSION:4\n#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"subs\",NAME=\"English\",DEFAULT=YES,AUTOSELECT=YES,FORCED=NO,LANGUAGE=\"en\",URI=\"{}\"\n#EXT-X-STREAM-INF:BANDWIDTH=5438980,AVERAGE-BANDWIDTH=2868620,SUBTITLES=\"subs\",\n{}", manifest_subs_path, manifest_path);
    let header = "EXTM3U\n#EXT-X-VERSION:4";
    let mut subtitles = String::new();
    if !no_subs {
        subtitles = format!("\n#EXT-X-MEDIA:TYPE=SUBTITLES,GROUP-ID=\"subs\",NAME=\"English\",DEFAULT=YES,AUTOSELECT=YES,FORCED=NO,LANGUAGE=\"en\",URI=\"{}\"", manifest_subs_path);
    }
    let manifest_str = format!(
        "\n#EXT-X-STREAM-INF:BANDWIDTH=5438980,AVERAGE-BANDWIDTH=2868620,SUBTITLES=\"subs\",\n{}",
        manifest_path
    );
    let str_to_write = format!("{}{}{}", header, subtitles, manifest_str);
    write!(file_buffer, "{}", str_to_write)?;
    Ok(())
}
