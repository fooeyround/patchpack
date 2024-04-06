use rayon::prelude::*;
use std::io;
use std::path::PathBuf;
use std::{io::prelude::*, path::Path};
use std::fs::File;
use liblzma::{decode_all, encode_all};
use tar::{Archive, Builder, Entries, Header};





pub struct RelativeBSPatch {
    content: Vec<u8>,
    path: PathBuf

}
impl RelativeBSPatch {
    pub fn new(entry: &mut tar::Entry<&[u8]>) -> io::Result<RelativeBSPatch> {
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        let path = entry.path()?.into_owned();
        return Ok(RelativeBSPatch {
            content: content.to_owned(),
            path
        });
    }
}



pub fn apply_patch(patch: &[u8], dest: &str) -> std::io::Result<()> {

    //Uncompress

    let uncompressed_patch = decode_all(patch)?;

        


    let mut archive = Archive::new(&uncompressed_patch[..]);


    //Unpack into memory


    let file_bspatches: Vec<tar::Entry<&[u8]>> =  archive.entries()?
        .filter_map(|p| p.ok()).collect();



    //Apply Said patches

    Ok(())

}



pub fn decode_patch(patch: &[u8]) -> std::io::Result<Vec<RelativeBSPatch>> {
    let uncompressed_patch = decode_all(patch)?;

    let mut archive = Archive::new(&uncompressed_patch[..]);


    //Unpack into memory


    let file_bspatches: Vec<RelativeBSPatch> =  archive.entries()?
        .filter_map(|p| {
            RelativeBSPatch::new(&mut p.ok()?).ok()
        }).collect();

    Ok(file_bspatches)
   
}



pub fn encode_patch(files: Vec<RelativeBSPatch>) -> std::io::Result<Vec<u8>> {



    let mut patch = Vec::new();

    let mut builder = Builder::new(&mut patch);


    for bs_patch in files {

        let mut header = Header::new_gnu();
        header.set_size(bs_patch.content.len() as u64);
        header.set_cksum();

        builder.append_data(&mut header,bs_patch.path.as_path(), &bs_patch.content[..])?;
    }
    
    
    
    let data = builder.into_inner()?.to_owned();
    
    encode_all(&data[..], 9)

}
