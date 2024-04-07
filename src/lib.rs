//! PatchPack handling library
//!
//! These consist of an LZMA compressed Tarball that contains a relative directory filled with a.b.bspatch files, a.b being the file it patches.
//!

use liblzma::{decode_all, encode_all};
use qbsdiff::{Bsdiff, Bspatch, ParallelScheme};
use rayon::prelude::*;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs, io};
use tar::{Archive, Builder, Header};
use walkdir::WalkDir;

//Currently does not work, I don't think I can conditionaly apply the macro, maybe I could if I had a fake empty macro to replace it with
#[cfg(feature = "logerrors")] use log::*; 
#[cfg(feature = "logerrors")] use log_derive::logfn;

pub struct RelativeBSPatch {
    content: Vec<u8>,
    path: PathBuf,
}
impl RelativeBSPatch {
    pub fn from_tar(entry: &mut tar::Entry<&[u8]>) -> io::Result<RelativeBSPatch> {
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        let path = entry.path()?.into_owned();
        return Self::new(path, content.to_owned());
    }

    pub fn new(path: PathBuf, content: Vec<u8>) -> io::Result<RelativeBSPatch> {
        return Ok(Self { content, path });
    }
}

///Given a patch and a destination, it will automatically patch relative to that target
///If the patch contains a `libs/engine.so.bspatch`, and dest is `app`, `app/libs/engine.so` will be patched.
pub fn apply_patch(patch: &[u8], dest: PathBuf) -> std::io::Result<Vec<isize>> {
    //LZMA Uncompress
    let file_patches = decode_patch(patch)?;

    let counts: Vec<isize> = file_patches
        .par_iter()
        .filter_map(|fp| {
            let mut rel_path = dest.join(&fp.path);

            println!("ext: {:?}", rel_path.extension()?); // TEMP TEST
            if rel_path.extension()? != "bspatch" {
                //If it is not a patch, don't apply it.
                return None;
            }
            rel_path.set_extension(""); //This removed the final .bspatch, giving the  actual file location.

            fs::create_dir_all(&rel_path).ok()?;

            let mut file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&rel_path)
                .ok()?;

            let mut original = Vec::new();

            file.read_to_end(&mut original).ok()?;

            let patch_applicator = Bspatch::new(&fp.content).ok()?;

            patch_applicator.apply(&original, &file).ok()?;

            Some(file.bytes().count() as isize - original.len() as isize)
        })
        .collect();

    Ok(counts)
}

fn bsdiff(source: &[u8], target: &[u8]) -> io::Result<Vec<u8>> {
    let mut patch = Vec::new();
    Bsdiff::new(source, target)
        .buffer_size(65536)
        .compression_level(7)
        .parallel_scheme(ParallelScheme::Never)
        .compare(io::Cursor::new(&mut patch))?;
    Ok(patch)
}

//I wonder if this should be split up into parts with a helper function after.
pub fn create_patch(before: PathBuf, after: PathBuf) -> io::Result<Vec<u8>> {
    // qbsdiff::Bsdiff::new(source, target);

    let bs_patches: Vec<RelativeBSPatch> = WalkDir::new(before)
        .into_iter()
        .zip(WalkDir::new(after).into_iter())
        .par_bridge()
        .filter_map(|e| Some((e.0.ok()?, e.1.ok()?)))
        .filter_map(|dir_entry| {
            assert!(dir_entry.0.path() == dir_entry.1.path());

            let mut file_path = dir_entry.0.into_path();

            let file_name = file_path.file_name()?;
            file_path.set_file_name(file_name.to_str()?.to_string() + ".bspatch");


            let mut old_content = Vec::new();
            let mut new_content = Vec::new();

            let mut old_file = fs::OpenOptions::new().read(true).open(&file_path).ok()?;
            let mut new_file = fs::OpenOptions::new().read(true).open(&file_path).ok()?;

            old_file.read_to_end(&mut old_content).ok()?;
            new_file.read_to_end(&mut new_content).ok()?;

            let content = bsdiff(&old_content, &new_content).ok()?;

            RelativeBSPatch::new(file_path, content).ok()
        })
        .collect();

    encode_patch(bs_patches)
}

pub fn build_patch(patch_dir: PathBuf) -> io::Result<Vec<u8>> {
    let bs_patches: Vec<RelativeBSPatch> = WalkDir::new(patch_dir)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter_map(|dir_entry| {
            let file_path = dir_entry.into_path();
            let mut content = Vec::new();

            let mut file = fs::OpenOptions::new().read(true).open(&file_path).ok()?;

            file.read_to_end(&mut content).ok()?;

            RelativeBSPatch::new(file_path, content).ok()
        })
        .collect();

    encode_patch(bs_patches)
}

/// Takes a patch's file contents as is.
pub fn decode_patch(patch: &[u8]) -> std::io::Result<Vec<RelativeBSPatch>> {
    let uncompressed_patch = decode_all(patch)?;

    let mut archive = Archive::new(&uncompressed_patch[..]);

    let file_bspatches: Vec<RelativeBSPatch> = archive
        .entries()?
        .filter_map(|p| RelativeBSPatch::from_tar(&mut p.ok()?).ok())
        .collect();

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

        builder.append_data(&mut header, bs_patch.path.as_path(), &bs_patch.content[..])?;
    }

    let data = builder.into_inner()?.to_owned();

    encode_all(&data[..], 9)
}

#[cfg(test)]
mod test {



    #[test]
    fn patch_build() {

    }
    

    #[test]
    fn patch_create() {

    }


    #[test]///This is less of a test, but a fake "network"
    fn patch_send() {

    }


    #[test]
    fn patch_apply() {

    }
    
    


}
