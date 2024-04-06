use qbsdiff::bsdiff;
use rayon::prelude::*;
use std::ffi::OsStr;
use std::{fs, io};
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



///Given a patch and a destination, it will automatically patch relative to that target
/// If the patch contains a `libs/engine.so.bspatch`, and dest is `app`, `app/libs/engine.so` will be patched. 
pub fn apply_patch(patch: &[u8], dest: PathBuf) -> std::io::Result<Vec<isize>> {

    //Uncompress

    let file_patches = decode_patch(patch)?;


    let counts: Vec<isize> = file_patches.par_iter().filter_map(|fp| {


        let mut rel_path = dest.join(&fp.path);

        println!("{:?}", rel_path.file_name().unwrap_or(OsStr::new("ThisIsABrokenFile_ReportThisAsABug")));

        fs::create_dir_all(&rel_path).ok()?;

        let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .open(&rel_path).ok()?;


        let mut original = Vec::new();

        file.read_to_end(&mut original).ok()?;


        let patch_applicator = qbsdiff::Bspatch::new(&fp.content).ok()?;

        patch_applicator.apply(&original, &file).ok()?;

        Some(file.bytes().count() as isize - original.len() as isize)

    }).collect();



    Ok(counts)

}


/// Takes a patch's file contents as is.
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

//returns the file contents that should be written to the file directly, this function *almost* an inverse to the one above.
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





#[cfg(test)]
mod test {



    #[test]
    fn patch_apply() {

    }

    #[test]
    fn patch_create() {
        
    }


    
}