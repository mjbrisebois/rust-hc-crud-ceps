use hdk::prelude::*;
use hc_crud::{
    now, get_origin_address, get_entities,
    create_entity, get_entity, update_entity, delete_entity,
    Entity, Collection, EntryModel, EntityType,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct GetEntityInput {
    pub id: EntryHash,
}

impl GetEntityInput {
    pub fn new(id: EntryHash) -> Self {
	GetEntityInput {
	    id: id,
	}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEntityInput<T> {
    pub addr: EntryHash,
    pub properties: T,
}

const TAG_POST: &'static str = "post";
const TAG_COMMENT: &'static str = "comment";


entry_defs![
    PostEntry::entry_def(),
    CommentEntry::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("Initialized 'happy_path' WASM");
    Ok(InitCallbackResult::Pass)
}


#[hdk_entry(id = "post", visibility="public")]
#[derive(Clone)]
pub struct PostEntry {
    pub title: String,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel for PostEntry {
    fn get_type(&self) -> EntityType {
	EntityType::new( "post", "entry" )
    }
}


#[hdk_entry(id = "comment", visibility="public")]
#[derive(Clone)]
pub struct CommentEntry {
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel for CommentEntry {
    fn get_type(&self) -> EntityType {
	EntityType::new( "comment", "entry" )
    }
}


// Post CRUD
#[hdk_extern]
pub fn create_post(mut input: PostEntry) -> ExternResult<Entity<PostEntry>> {
    if input.published_at.is_none() {
	input.published_at.replace( now()? );
    }

    debug!("Creating new post entry: {:?}", input );
    let entity = create_entity( &input )?;

    let pubkey = agent_info()?.agent_initial_pubkey;

    create_link(
	pubkey.into(),
	entity.id.clone(),
	LinkTag::new( TAG_POST )
    )?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_post(input: GetEntityInput) -> ExternResult<Entity<PostEntry>> {
    debug!("Get Post: {:?}", input.id );
    Ok( get_entity( &input.id )? )
}


#[hdk_extern]
pub fn update_post(mut input: UpdateEntityInput<PostEntry>) -> ExternResult<Entity<PostEntry>> {
    if input.properties.last_updated.is_none() {
	input.properties.last_updated.replace( now()? );
    }

    debug!("Updating post entry: {:?}", input.addr );
    let entity = update_entity( &input.addr, |previous: PostEntry, _| {
	let mut new_post = input.properties.clone();

	new_post.published_at = previous.published_at;

	Ok( new_post )
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn delete_post(input: GetEntityInput) -> ExternResult<HeaderHash> {
    debug!("Get Post: {:?}", input.id );
    Ok( delete_entity::<PostEntry>( &input.id )? )
}


// Comment CRUD
#[derive(Clone, Debug, Deserialize)]
pub struct CreateCommentInput {
    pub post_id: EntryHash,
    pub comment: CommentEntry,
}
#[hdk_extern]
pub fn create_comment(mut input: CreateCommentInput) -> ExternResult<Entity<CommentEntry>> {
    let post_id = get_origin_address( &input.post_id )?;

    // Check that the post exists and is not deleted
    get_post( GetEntityInput::new( post_id.clone() ) )?;

    if input.comment.published_at.is_none() {
	input.comment.published_at.replace( now()? );
    }

    debug!("Creating new comment entry: {:?}", input.comment );
    let entity = create_entity( &input.comment )?;

    create_link(
	post_id,
	entity.id.clone(),
	LinkTag::new( TAG_COMMENT )
    )?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_comment(input: GetEntityInput) -> ExternResult<Entity<CommentEntry>> {
    debug!("Get Post: {:?}", input.id );
    Ok( get_entity( &input.id )? )
}


#[hdk_extern]
pub fn get_comments_for_post(post_id: EntryHash) -> ExternResult<Collection<Entity<CommentEntry>>> {
    Ok( get_entities::<PostEntry, CommentEntry>( &post_id, LinkTag::new(TAG_COMMENT) )? )
}


#[hdk_extern]
pub fn update_comment(mut input: UpdateEntityInput<CommentEntry>) -> ExternResult<Entity<CommentEntry>> {
    if input.properties.last_updated.is_none() {
	input.properties.last_updated.replace( now()? );
    }

    debug!("Updating comment entry: {:?}", input.addr );
    let entity = update_entity( &input.addr, |previous: CommentEntry, _| {
	let mut new_comment = input.properties.clone();

	new_comment.published_at = previous.published_at;

	Ok( new_comment )
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn delete_comment(input: GetEntityInput) -> ExternResult<HeaderHash> {
    debug!("Get Comment: {:?}", input.id );
    Ok( delete_entity::<CommentEntry>( &input.id )? )
}
