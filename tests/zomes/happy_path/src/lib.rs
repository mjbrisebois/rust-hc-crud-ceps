use hdk::prelude::*;
use hc_crud::{
    now, get_origin_address, get_entities,
    create_entity, get_entity, update_entity, delete_entity,
    Entity, Collection, EntryModel, EntityType, // EmptyEntity,
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
    pub for_post: EntryHash,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel for CommentEntry {
    fn get_type(&self) -> EntityType {
	EntityType::new( "comment", "entry" )
    }
}

#[derive(Debug, Serialize)]
pub struct CommentInfo {
    pub for_post: Option<Entity<PostEntry>>,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel for CommentInfo {
    fn get_type(&self) -> EntityType {
	EntityType::new( "comment", "info" )
    }
}

impl CommentEntry {
    pub fn to_info(&self) -> CommentInfo {
	CommentInfo {
	    for_post: get_entity::<PostEntry>( &self.for_post ).ok(),
	    message: self.message.to_owned(),
	    published_at: self.published_at.to_owned(),
	    last_updated: self.last_updated.to_owned(),
	}
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

    entity.link_from( &pubkey.into(), TAG_POST.into() )?;

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
pub fn create_comment(mut input: CreateCommentInput) -> ExternResult<Entity<CommentInfo>> {
    let post_id = get_origin_address( &input.post_id )?;

    // Check that the post exists and is not deleted
    get_post( GetEntityInput::new( post_id.clone() ) )?;

    if input.comment.published_at.is_none() {
	input.comment.published_at.replace( now()? );
    }

    debug!("Creating new comment entry: {:?}", input.comment );
    let entity = create_entity( &input.comment )?
	.change_model( |comment| comment.to_info() );

    entity.link_from( &post_id, TAG_COMMENT.into() )?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_comment(input: GetEntityInput) -> ExternResult<Entity<CommentInfo>> {
    debug!("Get Post: {:?}", input.id );
    Ok(
	get_entity::<CommentEntry>( &input.id )?
	    .change_model( |comment| comment.to_info() )
    )
}


#[hdk_extern]
pub fn get_comments_for_post(post_id: EntryHash) -> ExternResult<Collection<Entity<CommentEntry>>> {
    Ok( get_entities::<PostEntry, CommentEntry, >( &post_id, TAG_COMMENT.into() )? )
}


// This method is for one of the failure tests; that is why it doesn't make logical sense.
#[hdk_extern]
pub fn get_posts_for_comment(comment_id: EntryHash) -> ExternResult<Collection<Entity<PostEntry>>> {
    Ok( get_entities::<CommentEntry, PostEntry, >( &comment_id, TAG_COMMENT.into() )? )
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


#[derive(Clone, Debug, Deserialize)]
pub struct MoveCommentInput {
    pub comment_addr: EntryHash,
    pub post_id: EntryHash,
}
#[hdk_extern]
pub fn move_comment_to_post (input: MoveCommentInput) -> ExternResult<Entity<CommentEntry>> {
    let mut current_base = input.post_id.clone();
    let new_base = input.post_id.clone();

    let entity = update_entity( &input.comment_addr, |mut previous: CommentEntry, _| {
	current_base = previous.for_post;
	previous.for_post = new_base.to_owned();

	Ok( previous )
    })?;

    debug!("Delinking previous base to ENTRY: {:?}", current_base );
    entity.move_link_from( TAG_COMMENT.into(), &current_base, &new_base )?;

    Ok( entity )
}
