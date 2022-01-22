pub struct PageCache {}

/*use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    hash::{Hash, Hasher},
    ptr::NonNull,
};

use narcissus_core::{align_offset, virtual_commit, virtual_free, virtual_reserve, Uuid};
use std::ffi::c_void;

const ID_INDEX_BITS: u32 = 22;
const ID_GEN_BITS: u32 = 10;

const PAGE_SIZE: usize = 4096;
const MAX_FIELD_TYPES: usize = 256;

// SAFETY: Blittable - no padding, zero valid, blah blah.
pub unsafe trait FieldType {
    const UUID: Uuid;
    fn version() -> u32;
    fn name() -> &'static str;
}

#[derive(Clone, Copy)]
pub struct Id(u32);

impl Id {
    pub const fn null() -> Self {
        Self(0)
    }
}

#[derive(Clone, Copy)]
pub struct Block(u32);

impl Block {
    pub const fn null() -> Self {
        Self(0)
    }
}

pub struct Config {
    version: u32,
    block_size: u32,
    block_cap: u32,
    id_cap: u32,
}

impl Config {
    fn calculate_mapping(&self) -> Mapping {
        const NUM_GUARD_PAGES: usize = 4;

        let mut size = 0;

        let id_info_offset = size;
        let id_info_len = self.id_cap as usize * std::mem::size_of::<IdInfo>();

        size += id_info_len;
        size = align_offset(size, PAGE_SIZE);
        size += NUM_GUARD_PAGES * PAGE_SIZE;

        let block_info_offset = size;
        let block_info_len = self.block_cap as usize * std::mem::size_of::<BlockInfo>();

        size += block_info_len;
        size = align_offset(size, PAGE_SIZE);
        size += NUM_GUARD_PAGES * PAGE_SIZE;

        let block_storage_offset = size;
        let block_storage_len = self.block_cap as usize * self.block_size as usize;

        size += block_storage_len;
        size = align_offset(size, PAGE_SIZE);
        size += NUM_GUARD_PAGES * PAGE_SIZE;

        Mapping {
            size,
            id_info_offset,
            id_info_len,
            block_info_offset,
            block_info_len,
            block_storage_offset,
            block_storage_len,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 0,
            block_size: 16 * 1024, // 16KiB blocks
            block_cap: 262_144,    // 4GiB in total
            id_cap: 4_194_304,
        }
    }
}

struct Mapping {
    size: usize,
    id_info_offset: usize,
    id_info_len: usize,
    block_info_offset: usize,
    block_info_len: usize,
    block_storage_offset: usize,
    block_storage_len: usize,
}

struct IdInfo {
    index_and_generation: u32,
    index_in_block: u32,
    block: Block,
}

struct BlockInfo {
    num_things: u32,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct FieldTypes([u64; MAX_FIELD_TYPES / 64]);

impl FieldTypes {
    pub fn new() -> Self {
        Self([0; MAX_FIELD_TYPES / 64])
    }

    pub fn set(&mut self, field_type_index: FieldTypeIndex) {
        let field_type_index = field_type_index.0;
        let index = field_type_index / (MAX_FIELD_TYPES / 64);
        self.0;
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct Descriptor {
    shift: u32,
    offset: u32,
    stride: u32,
    width: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Schema {
    hash: [u8; 32],
    field_types: FieldTypes,
    cap: usize,
    descriptors: Box<[Descriptor]>,
}

impl Schema {
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl Hash for Schema {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

enum SchemaFieldMode {
    Scalar,
    Vector,
}

struct SchemaField {
    uuid: Uuid,
    size: usize,
    align: usize,
    mode: SchemaFieldMode,
}

pub struct SchemaBuilder<'cfg, 'reg> {
    config: &'cfg Config,
    registry: &'reg Registry,
    indexed: bool,
    fields: Vec<SchemaField>,
}

impl<'cfg, 'reg> SchemaBuilder<'cfg, 'reg> {
    pub fn new(config: &'cfg Config, registry: &'reg Registry) -> Self {
        Self {
            config,
            registry,
            indexed: false,
            fields: Vec::new(),
        }
    }

    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    pub fn add_vector_field<T: FieldType>(mut self) -> Self {
        self.fields.push(SchemaField {
            uuid: T::UUID,
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            mode: SchemaFieldMode::Vector,
        });
        self
    }

    pub fn add_scalar_field<T: FieldType>(mut self) -> Self {
        self.fields.push(SchemaField {
            uuid: T::UUID,
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            mode: SchemaFieldMode::Scalar,
        });
        self
    }

    // fn push(&mut self, type_id: TypeId, field: &SchemaField) {
    //     self.descriptor_indices
    //         .insert(type_id, DescriptorIndex(self.descriptors.len()));
    //     self.fields.push(Descriptor {
    //         shift: 0,
    //         offset: self.offset,
    //         stride: 0,
    //         width: field.size,
    //     });
    //     self.offset += field.size;
    // }

    // fn push_shift(&mut self, type_id: TypeId, part_desc: &PartDesc, shift: usize) {
    //     self.descriptor_indices
    //         .insert(type_id, DescriptorIndex(self.descriptors.len()));
    //     self.descriptors.push(Descriptor {
    //         shift,
    //         offset: self.offset,
    //         stride: part_desc.size,
    //         width: part_desc.size,
    //     });
    //     self.offset += part_desc.size << shift;
    // }

    // fn push_vector(&mut self, type_id: TypeId, part_desc: &PartDesc, count: usize) {
    //     self.descriptor_indices
    //         .insert(type_id, DescriptorIndex(self.descriptors.len()));
    //     self.descriptors.push(Descriptor {
    //         shift: 0,
    //         offset: self.offset,
    //         stride: part_desc.size,
    //         width: part_desc.size,
    //     });
    //     self.offset += part_desc.size * count;
    // }

    fn build(&self) -> Schema {
        fn align_offset(x: usize, align: usize) -> usize {
            debug_assert!(align.is_power_of_two());
            (x + align - 1) & !(align - 1)
        }

        let mut field_types = FieldTypes::new();
        let mut offset = 0;
        let mut descriptors = Vec::new();

        for field in &self.fields {
            let field_type_index = self.registry.get_field_type_index_val(&field.uuid);
        }

        let field_type_indices = self
            .fields
            .iter()
            .map(|field| self.registry.get_field_type_index_val(&field.uuid))
            .collect::<Vec<_>>();

        let fields = self
            .fields
            .iter()
            .filter(|field| field.size != 0)
            .collect::<Vec<_>>();

        Schema {
            hash: [0; 32],
            field_types: FieldTypes([0; MAX_FIELD_TYPES / 64]),
            cap: 0,
            descriptors: descriptors.into_boxed_slice(),
        }
    }

    pub fn build_aos(self) -> Schema {
        Schema {
            hash: [0; 32],
            field_types: FieldTypes([0; MAX_FIELD_TYPES / 64]),
            cap: 0,
            descriptors: Box::new([]),
        }
    }

    pub fn build_soa(self) -> Schema {
        Schema {
            hash: [0; 32],
            field_types: FieldTypes([0; MAX_FIELD_TYPES / 64]),
            cap: 0,
            descriptors: Box::new([]),
        }
    }

    pub fn build_aosoa(self, stride: usize) -> Schema {
        assert!(stride.is_power_of_two());
        Schema {
            hash: [0; 32],
            field_types: FieldTypes([0; MAX_FIELD_TYPES / 64]),
            cap: 0,
            descriptors: Box::new([]),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FieldTypeIndex(usize);

impl FieldTypeIndex {
    fn new(index: usize) -> Self {
        assert!(index < MAX_FIELD_TYPES);
        Self(index)
    }
}

struct FieldTypeInfo {
    debug_name: &'static str,
    align: usize,
    size: usize,
}

pub struct Registry {
    field_lookup: HashMap<Uuid, FieldTypeIndex>,
    field_types: Vec<FieldTypeInfo>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            field_lookup: HashMap::new(),
            field_types: Vec::new(),
        }
    }

    pub fn register_field_type<T: FieldType>(&mut self) {
        let type_uuid = T::UUID;
        let debug_name = T::name();
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        if let Entry::Vacant(entry) = self.field_lookup.entry(type_uuid) {
            let next_index = self.field_types.len();
            entry.insert(FieldTypeIndex::new(next_index));
            self.field_types.push(FieldTypeInfo {
                debug_name,
                align,
                size,
            });
        } else {
            panic!("don't register field types more than once");
        }
    }

    fn get_field_type_index<T: FieldType>(&self) -> FieldTypeIndex {
        *self
            .field_lookup
            .get(&T::UUID)
            .expect("failed to find field type")
    }

    fn get_field_type_index_val(&self, uuid: &Uuid) -> FieldTypeIndex {
        *self
            .field_lookup
            .get(uuid)
            .expect("failed to find field type")
    }
}

pub struct World<'cfg, 'reg> {
    config: &'cfg Config,
    registry: &'reg Registry,

    schemas: Vec<Schema>,

    free_ids: VecDeque<u32>,
    id_info_len: usize,
    id_info: NonNull<IdInfo>,

    free_blocks: Vec<u32>,
    block_info_len: usize,
    block_info: NonNull<BlockInfo>,
    block_storage: NonNull<u8>,

    map: *mut c_void,
    map_size: usize,
}

impl<'cfg, 'reg> World<'cfg, 'reg> {
    pub fn new(config: &'cfg Config, registry: &'reg Registry) -> Self {
        let mapping = config.calculate_mapping();

        let map = unsafe { virtual_reserve(mapping.size) };
        assert!(!map.is_null());

        let id_info = NonNull::new(unsafe {
            std::mem::transmute::<*mut c_void, *mut IdInfo>(
                map.offset(mapping.id_info_offset as isize),
            )
        })
        .unwrap();

        let block_info = NonNull::new(unsafe {
            std::mem::transmute::<*mut c_void, *mut BlockInfo>(
                map.offset(mapping.block_info_offset as isize),
            )
        })
        .unwrap();

        // Always commit the entire info area, it's not all that much memory.
        unsafe { virtual_commit(block_info.as_ptr() as *mut c_void, mapping.block_info_len) };

        let block_storage = NonNull::new(unsafe {
            std::mem::transmute::<*mut c_void, *mut u8>(
                map.offset(mapping.block_storage_offset as isize),
            )
        })
        .unwrap();

        Self {
            config,
            registry,
            schemas: Vec::new(),
            free_ids: VecDeque::new(),
            id_info_len: 0,
            id_info,
            free_blocks: Vec::new(),
            block_info_len: 0,
            block_info,
            block_storage,
            map,
            map_size: mapping.size,
        }
    }

    #[inline]
    fn id_infos(&self) -> &[IdInfo] {
        unsafe { std::slice::from_raw_parts(self.id_info.as_ptr(), self.id_info_len) }
    }

    #[inline]
    fn id_infos_mut(&mut self) -> &mut [IdInfo] {
        unsafe { std::slice::from_raw_parts_mut(self.id_info.as_ptr(), self.id_info_len) }
    }

    #[inline]
    fn block_infos(&self) -> &[BlockInfo] {
        unsafe { std::slice::from_raw_parts(self.block_info.as_ptr(), self.block_info_len) }
    }

    #[inline]
    fn block_infos_mut(&mut self) -> &mut [BlockInfo] {
        unsafe { std::slice::from_raw_parts_mut(self.block_info.as_ptr(), self.block_info_len) }
    }

    #[inline]
    fn block_storage(&self, block: Block) -> &[u8] {
        let block_index = block.0 as usize;
        assert!(block_index < self.block_info_len);
        let block_size = self.config.block_size as usize;
        unsafe {
            std::slice::from_raw_parts(
                self.block_storage
                    .as_ptr()
                    .offset((block_index * block_size) as isize),
                block_size,
            )
        }
    }

    #[inline]
    fn block_storage_mut(&mut self, block: Block) -> &mut [u8] {
        let block_index = block.0 as usize;
        assert!(block_index < self.block_info_len);
        let block_size = self.config.block_size as usize;
        unsafe {
            std::slice::from_raw_parts_mut(
                self.block_storage
                    .as_ptr()
                    .offset((block_index * block_size) as isize),
                block_size,
            )
        }
    }

    pub fn allocate_ids(&self, ids: &mut [Id]) {}
    pub fn release_ids(&self, ids: &[Id]) {}

    pub fn allocate_blocks(&self, blocks: &mut [Block]) {}
    pub fn release_blocks(&self, blocks: &[Block]) {}

    pub fn insert_blocks(&self, schema: &Schema, blocks: &[Block]) {}
    pub fn insert_blocks_indexed(&self, schema: &Schema, blocks: &[Block], ids: &[Id]) {}
}

impl<'cfg, 'reg> Drop for World<'cfg, 'reg> {
    fn drop(&mut self) {
        unsafe { virtual_free(self.map, self.map_size) };
    }
}

// mod app;
// //mod gfx;
// mod world;

// use app::App;
// use narcissus_core::{FixedVec, Uuid};
// use world::{Block, Config, FieldType, Id, Registry, SchemaBuilder, World};

// // Units

// #[derive(Clone, Copy)]
// struct WorldX(f32);
// unsafe impl FieldType for WorldX {
//     const UUID: Uuid = Uuid::parse_str_unwrap("fa565a43-4ec0-460d-84bb-32ef861ff48b");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "World Position X"
//     }
// }

// #[derive(Clone, Copy)]
// struct WorldY(f32);
// unsafe impl FieldType for WorldY {
//     const UUID: Uuid = Uuid::parse_str_unwrap("b7e3ccbf-d839-4ee0-8be4-b068a25a299f");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "World Position Y"
//     }
// }

// #[derive(Clone, Copy)]
// struct WorldZ(f32);
// unsafe impl FieldType for WorldZ {
//     const UUID: Uuid = Uuid::parse_str_unwrap("a6bcf557-3117-4664-ae99-8c8ceb96467c");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "World Position Z"
//     }
// }

// #[derive(Clone, Copy)]
// struct Health(f32);
// unsafe impl FieldType for Health {
//     const UUID: Uuid = Uuid::parse_str_unwrap("f3c7be8f-2120-42bd-a0be-bfe26d95198b");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "Health"
//     }
// }

// #[derive(Clone, Copy)]
// struct Armor(f32);
// unsafe impl FieldType for Armor {
//     const UUID: Uuid = Uuid::parse_str_unwrap("42fde8e0-7576-4039-8169-769e68aafe8b");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "Armor"
//     }
// }

// struct Ship();
// unsafe impl FieldType for Ship {
//     const UUID: Uuid = Uuid::parse_str_unwrap("02024fee-ef95-42c3-877b-aa9f6afbf0a2");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "Ship"
//     }
// }

// struct Asteroid();
// unsafe impl FieldType for Asteroid {
//     const UUID: Uuid = Uuid::parse_str_unwrap("22e8e546-0aeb-4e23-beee-89075522fb57");

//     fn version() -> u32 {
//         0
//     }

//     fn name() -> &'static str {
//         "Asteroid"
//     }
// }

// pub fn main() {
//     let app = App::new();

//     let window = app.create_window();

//     let mut registry = Registry::new();
//     registry.register_field_type::<WorldX>();
//     registry.register_field_type::<WorldY>();
//     registry.register_field_type::<WorldZ>();
//     registry.register_field_type::<Health>();
//     registry.register_field_type::<Armor>();
//     registry.register_field_type::<Ship>();
//     registry.register_field_type::<Asteroid>();

//     let config = Config::default();
//     let world = World::new(&config, &registry);

//     const NUM_SHIPS: usize = 1_000;
//     const NUM_ASTEROIDS: usize = 100_000;

//     let ships_schema = SchemaBuilder::new(&config, &registry)
//         .indexed()
//         .add_vector_field::<WorldX>()
//         .add_vector_field::<WorldY>()
//         .add_vector_field::<WorldZ>()
//         .add_vector_field::<Health>()
//         .add_vector_field::<Armor>()
//         .add_scalar_field::<Ship>()
//         .build_aosoa(4);

//     let mut ship_blocks = FixedVec::<_, 16>::new();
//     ship_blocks.resize(NUM_SHIPS / ships_schema.capacity(), Block::null());
//     world.allocate_blocks(&mut ship_blocks);

//     let mut ship_ids = vec![Id::null(); NUM_SHIPS];
//     world.allocate_ids(&mut ship_ids);

//     let make_ship_scalars = |ship: &mut Ship| {};

//     let make_ship = |world_x: &mut WorldX,
//                      world_y: &mut WorldY,
//                      world_z: &mut WorldZ,
//                      health: &mut Health,
//                      armor: &mut Armor| {};

//     {
//         let mut block_iter = ship_blocks.iter();
//         let mut ids_iter = ship_ids.chunks(ships_schema.capacity());
//         for (block, ids) in block_iter.zip(ids_iter) {
//             //make_ship_scalars();
//             for id in ids {
//                 //make_ship();
//             }
//         }
//     }

//     world.insert_blocks_indexed(&ships_schema, &ship_blocks, &ship_ids);

//     let asteroids_schema = SchemaBuilder::new(&config, &registry)
//         .add_vector_field::<WorldX>()
//         .add_vector_field::<WorldY>()
//         .add_vector_field::<WorldZ>()
//         .add_vector_field::<Health>()
//         .add_scalar_field::<Asteroid>()
//         .build_aosoa(4);

//     let mut asteroid_blocks = vec![Block::null(); NUM_ASTEROIDS / asteroids_schema.capacity()];
//     world.allocate_blocks(&mut asteroid_blocks);

//     world.insert_blocks(&asteroids_schema, &asteroid_blocks);

//     app.destroy_window(window);
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

*/
