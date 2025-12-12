extern crate rand;

use std::sync::atomic::{AtomicUsize, Ordering};

static CPY_CNT: AtomicUsize = AtomicUsize::new(0);

fn copy(src: &[u8], dst: &mut [u8]) {
    assert_eq!(src.len(), dst.len());

    CPY_CNT.fetch_add(1, Ordering::AcqRel);

    dst.copy_from_slice(src);
}
fn lut2(table: &[u8], from: &[u8], to: &mut [u8]) {
    assert_eq!(from.len(), 4);
    let lo = from[0];
    let hi = from[1];
    let k = hi as usize * 256 + lo as usize;
    copy(&table[k..k + 1], to);
}
fn lut1(table: &[u8], from: &[u8], to: &mut [u8]) {
    assert_eq!(from.len(), 4);
    let lo = from[0] as usize;
    copy(&table[lo..lo + 1], to);
}

fn add_u32(a: u32, b: u32) -> u32 {
    const LUT: ([u8; 65536], [u8; 65536]) = const {
        let mut x = [0u8; 65536];
        let mut y = [0u8; 65536];
        let mut i = 0usize;
        while i < 256 {
            let mut j = 0usize;
            while j < 256 {
                let k = i * 256 + j;
                let overflows: bool;
                (x[k], overflows) = (i as u8).overflowing_add(j as u8);
                y[k] = if overflows { 1 } else { 0 };
                j += 1;
            }
            i += 1;
        }
        (x, y)
    };

    let lhs = a.to_le_bytes();
    let rhs = b.to_le_bytes();

    let mut tsum0 = [0u8; 4];
    let mut tsum1 = [0u8; 4];
    let mut tsum2 = [0u8; 4];
    let mut tcar0 = [0u8; 4];
    let mut tcar1 = [0u8; 4];
    let mut out = [0u8; 4];

    // out0, [sm10] <- lhs0, rhs0
    copy(&rhs[0..1], &mut tsum0[0..1]);
    copy(&lhs[0..1], &mut tsum0[1..2]);
    lut2(&LUT.0, &tsum0[0..4], &mut out[0..1]);
    copy(&tsum0[0..2], &mut tcar0[0..2]);
    lut2(&LUT.1, &tcar0[0..4], &mut tsum1[0..1]);

    // [sm00], [cr20] <- lhs1, [sm10]
    // out1, [cr21] <- rhs1, [sm00]
    // [sm10], _ <- [cr21], [cr20]
    copy(&lhs[1..2], &mut tsum1[1..2]);
    lut2(&LUT.0, &tsum1[0..4], &mut tsum0[0..1]);
    copy(&tsum1[0..2], &mut tcar1[0..2]);
    lut2(&LUT.1, &tcar1[0..4], &mut tsum2[0..1]);

    copy(&rhs[1..2], &mut tsum0[1..2]);
    lut2(&LUT.0, &tsum0[0..4], &mut out[1..2]);
    copy(&tsum0[0..2], &mut tcar0[0..2]);
    lut2(&LUT.1, &tcar0[0..4], &mut tsum2[1..2]);
    lut2(&LUT.0, &tsum2[0..4], &mut tsum1[0..1]);

    // t, c1 <- lhs2, [sm10]
    // o2, c0 <- t, rhs2
    // c2, _ <- c0, c1
    copy(&lhs[2..3], &mut tsum1[1..2]);
    lut2(&LUT.0, &tsum1[0..4], &mut tsum0[0..1]);
    copy(&tsum1[0..2], &mut tcar1[0..2]);
    lut2(&LUT.1, &tcar1[0..4], &mut tsum2[0..1]);

    copy(&rhs[2..3], &mut tsum0[1..2]);
    lut2(&LUT.0, &tsum0[0..4], &mut out[2..3]);
    copy(&tsum0[0..2], &mut tcar0[0..2]);
    lut2(&LUT.1, &tcar0[0..4], &mut tsum2[1..2]);
    lut2(&LUT.0, &tsum2[0..4], &mut tsum1[0..1]);

    // t, _ <- lhs3, [sm10]
    // o3, _ <- t, rhs3
    copy(&lhs[3..4], &mut tsum1[1..2]);
    lut2(&LUT.0, &tsum1[0..4], &mut tsum0[0..1]);

    copy(&rhs[3..4], &mut tsum0[1..2]);
    lut2(&LUT.0, &tsum0[0..4], &mut out[3..4]);

    u32::from_le_bytes(out)
}

fn xor_u32(a: u32, b: u32) -> u32 {
    const LUT: [u8; 65536] = const {
        let mut x = [0u8; 65536];
        let mut i = 0;
        while i < 256 {
            let mut j = 0;
            while j < 256 {
                x[i * 256 + j] = (i as u8) ^ (j as u8);
                j += 1;
            }
            i += 1;
        }
        x
    };

    let lhs = a.to_le_bytes();
    let rhs = b.to_le_bytes();

    let mut txor = [0u8; 4];
    let mut out = [0u8; 4];

    copy(&lhs[0..1], &mut txor[0..1]);
    copy(&rhs[0..1], &mut txor[1..2]);
    lut2(&LUT, &txor[0..4], &mut out[0..1]);
    copy(&lhs[1..2], &mut txor[0..1]);
    copy(&rhs[1..2], &mut txor[1..2]);
    lut2(&LUT, &txor[0..4], &mut out[1..2]);
    copy(&lhs[2..3], &mut txor[0..1]);
    copy(&rhs[2..3], &mut txor[1..2]);
    lut2(&LUT, &txor[0..4], &mut out[2..3]);
    copy(&lhs[3..4], &mut txor[0..1]);
    copy(&rhs[3..4], &mut txor[1..2]);
    lut2(&LUT, &txor[0..4], &mut out[3..4]);

    u32::from_le_bytes(out)
}

const LUT_SL: ([u8; 65536], [u8; 65536]) = const {
    let mut x = [0u8; 65536];
    let mut y = [0u8; 65536];
    let mut i = 0;
    while i < 256 {
        let mut jj = 0;
        while jj < 256 {
            let j = jj % 8;
            let z = (i << j) as u16;
            x[i * 256 + jj] = (z >> 8) as u8;
            y[i * 256 + jj] = (z >> 0) as u8;
            jj += 1;
        }
        i += 1;
    }
    (x, y)
};
const LUT_OR: [u8; 65536] = const {
    let mut x = [0u8; 65536];
    let mut i = 0;
    while i < 256 {
        let mut j = 0;
        while j < 256 {
            x[i * 256 + j] = i as u8 | j as u8;
            j += 1;
        }
        i += 1;
    }
    x
};
const LUT_AND: [u8; 65536] = const {
    let mut x = [0u8; 65536];
    let mut i = 0;
    while i < 256 {
        let mut j = 0;
        while j < 256 {
            x[i * 256 + j] = i as u8 & j as u8;
            j += 1;
        }
        i += 1;
    }
    x
};

const LUT_SL_SHUF: [[u8; 256]; 4] = const {
    let mut x = [[0u8; 256]; 4];
    let mut i = 0;
    // i is new idx, jj is shamt
    // so if i = 1, mask(jj)=9, then j=1, and x[i][jj] = 0
    while i < 4 {
        let mut jj = 0;
        while jj < 256 {
            let j = (jj % 32) / 8;
            assert!(i + j < 8);
            x[i][jj] = (4 + i - j) as u8;
            jj += 1;
        }
        i += 1;
    }
    x
};

fn sll_u32(a: u32, b: u8) -> u32 {
    // correct for shamt<8
    let lhs = a.to_le_bytes();
    let shamt = b;

    let mut tslh = [0u8; 4];
    let mut tsll = [0u8; 4];
    let mut tjxn0 = [0u8; 4];
    let mut tjxn1 = [0u8; 4];
    let mut tjxn2 = [0u8; 4];
    let mut shbf = [0u8; 8];
    let mut tshf0 = [0u8; 4];
    let mut tshf1 = [0u8; 4];
    let mut tshf2 = [0u8; 4];
    let mut tshf3 = [0u8; 4];
    let mut tshbf = [0u8; 4];
    let mut out = [0u8; 4];

    // tsl goes (x, shamt)

    copy(&[shamt], &mut tsll[0..1]);

    copy(&lhs[0..1], &mut tsll[1..2]);
    lut2(&LUT_SL.1, &tsll[0..4], &mut tjxn0[1..2]);
    lut2(&LUT_OR, &tjxn0[0..4], &mut shbf[4..5]);
    copy(&tsll[0..2], &mut tslh[0..2]);
    lut2(&LUT_SL.0, &tslh[0..4], &mut tjxn1[0..1]);

    copy(&lhs[1..2], &mut tsll[1..2]);
    lut2(&LUT_SL.1, &tsll[0..4], &mut tjxn1[1..2]);
    lut2(&LUT_OR, &tjxn1[0..4], &mut shbf[5..6]);
    copy(&lhs[1..2], &mut tslh[1..2]);
    lut2(&LUT_SL.0, &tslh[0..4], &mut tjxn1[0..1]);

    copy(&lhs[2..3], &mut tsll[1..2]);
    lut2(&LUT_SL.1, &tsll[0..4], &mut tjxn1[1..2]);
    lut2(&LUT_OR, &tjxn1[0..4], &mut shbf[6..7]);
    copy(&lhs[2..3], &mut tslh[1..2]);
    lut2(&LUT_SL.0, &tslh[0..4], &mut tjxn2[0..1]);

    copy(&lhs[3..4], &mut tsll[1..2]);
    lut2(&LUT_SL.1, &tsll[0..4], &mut tjxn2[1..2]);
    lut2(&LUT_OR, &tjxn2[0..4], &mut shbf[7..8]);

    copy(&[shamt], &mut tshf0[0..1]);
    lut1(&LUT_SL_SHUF[0], &tshf0[0..4], &mut tshbf[0..1]);
    lut1(&shbf, &tshbf[0..4], &mut out[0..1]);
    copy(&[shamt], &mut tshf1[0..1]);
    lut1(&LUT_SL_SHUF[1], &tshf1[0..4], &mut tshbf[0..1]);
    lut1(&shbf, &tshbf[0..4], &mut out[1..2]);
    copy(&[shamt], &mut tshf2[0..1]);
    lut1(&LUT_SL_SHUF[2], &tshf2[0..4], &mut tshbf[0..1]);
    lut1(&shbf, &tshbf[0..4], &mut out[2..3]);
    copy(&[shamt], &mut tshf3[0..1]);
    lut1(&LUT_SL_SHUF[3], &tshf3[0..4], &mut tshbf[0..1]);
    lut1(&shbf, &tshbf[0..4], &mut out[3..4]);

    u32::from_le_bytes(out)
}

fn cteq_u32(a: u32, b: u32) -> u32 {
    const LUT: [u8; 65536] = const {
        let mut x = [0u8; 65536];
        let mut i = 0;
        while i < 256 {
            let mut j = 0;
            while j < 256 {
                x[i * 256 + j] = if i == j { 1 } else { 0 };
                j += 1;
            }
            i += 1;
        }
        x
    };

    let lhs = a.to_le_bytes();
    let rhs = b.to_le_bytes();

    let mut teq = [0u8; 4];
    let mut tand0 = [0u8; 4];
    let mut tand1 = [0u8; 4];
    let mut out = [0u8; 4];

    copy(&lhs[0..1], &mut teq[0..1]);
    copy(&rhs[0..1], &mut teq[1..2]);
    lut2(&LUT, &teq[0..4], &mut tand0[0..1]);
    copy(&lhs[1..2], &mut teq[0..1]);
    copy(&rhs[1..2], &mut teq[1..2]);
    lut2(&LUT, &teq[0..4], &mut tand0[1..2]);
    lut2(&LUT_AND, &tand0[0..4], &mut tand1[0..1]);
    copy(&lhs[2..3], &mut teq[0..1]);
    copy(&rhs[2..3], &mut teq[1..2]);
    lut2(&LUT, &teq[0..4], &mut tand1[1..2]);
    lut2(&LUT_AND, &tand1[0..4], &mut tand0[0..1]);
    copy(&lhs[3..4], &mut teq[0..1]);
    copy(&rhs[3..4], &mut teq[1..2]);
    lut2(&LUT, &teq[0..4], &mut tand0[1..2]);
    lut2(&LUT_AND, &tand0[0..4], &mut out[0..1]);

    u32::from_le_bytes(out)
}

fn compare(op: &str, op1: impl Fn(u32, u32) -> u32, op2: impl Fn(u32, u32) -> u32) {
    CPY_CNT.store(0, Ordering::Release);
    let rounds = 10_000_000;
    for _ in 0..rounds {
        let a: u32 = rand::random();
        let b: u32 = rand::random();
        let correct = op1(a, b);
        let got = op2(a, b);
        if correct != got {
            println!("{a:08x} + {b:08x} = {correct:08x} vs. {got:08x}");
        }
    }
    println!("{op} done in {}", CPY_CNT.load(Ordering::Acquire) / rounds);
}

pub fn main() {
    compare("add32", |a, b| a.wrapping_add(b), add_u32);
    compare("xor32", |a, b| a ^ b, xor_u32);
    compare(
        "sll32",
        |a, b| a.wrapping_shl(b),
        |a, b| sll_u32(a, b as u8),
    );
    compare("ceq32", |a, b| if a == b { 1 } else { 0 }, cteq_u32);
}
