//! Shannon entropy filtering for secret validation
//!
//! This module implements entropy-based validation of potential secrets using statistical
//! analysis to distinguish random strings (likely secrets) from non-random text.
//! 
//! Uses the proven entropy algorithms from scan-v2 with optimizations for v3 architecture.

use crate::scan_v3::{
    data::SecretMatch,
    filters::{Filter}
};
use anyhow::Result;
use regex::bytes::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Pre-compiled regex patterns for entropy analysis (zero-copy shared across threads)
///
/// These regexes are expensive to compile and immutable, so we compile them once
/// and share across all entropy analysis calls via Arc for zero-copy access.
static STATIC_HEX_REGEX: LazyLock<Arc<Regex>> = LazyLock::new(|| {
    Arc::new(Regex::new("^[0-9a-fA-F]{16,}$").expect("Failed to compile hex regex"))
});

static STATIC_CAP_AND_NUMBERS_REGEX: LazyLock<Arc<Regex>> = LazyLock::new(|| {
    Arc::new(Regex::new("^[0-9A-Z]{16,}$").expect("Failed to compile caps+numbers regex"))
});

/// Pre-computed bigram set for fast lookup (zero-copy shared across threads)
///
/// This contains 488 common bigrams found in source code. Computing this set
/// on every entropy call is wasteful - better to compute once and share via Arc.
static STATIC_BIGRAMS: LazyLock<Arc<HashSet<&'static [u8]>>> = LazyLock::new(|| {
    // 488 common bigrams found in source code (from ripsecrets research)
    let bigrams_bytes = b"er,te,an,en,ma,ke,10,at,/m,on,09,ti,al,io,.h,./,..,ra,ht,es,or,tm,pe,ml,re,in,3/,n3,0F,ok,ey,00,80,08,ss,07,15,81,F3,st,52,KE,To,01,it,2B,2C,/E,P_,EY,B7,se,73,de,VP,EV,to,od,B0,0E,nt,et,_P,A0,60,90,0A,ri,30,ar,C0,op,03,ec,ns,as,FF,F7,po,PK,la,.p,AE,62,me,F4,71,8E,yp,pa,50,qu,D7,7D,rs,ea,Y_,t_,ha,3B,c/,D2,ls,DE,pr,am,E0,oc,06,li,do,id,05,51,40,ED,_p,70,ed,04,02,t.,rd,mp,20,d_,co,ro,ex,11,ua,nd,0C,0D,D0,Eq,le,EF,wo,e_,e.,ct,0B,_c,Li,45,rT,pt,14,61,Th,56,sT,E6,DF,nT,16,85,em,BF,9E,ne,_s,25,91,78,57,BE,ta,ng,cl,_t,E1,1F,y_,xp,cr,4F,si,s_,E5,pl,AB,ge,7E,F8,35,E2,s.,CF,58,32,2F,E7,1B,ve,B1,3D,nc,Gr,EB,C6,77,64,sl,8A,6A,_k,79,C8,88,ce,Ex,5C,28,EA,A6,2A,Ke,A7,th,CA,ry,F0,B6,7/,D9,6B,4D,DA,3C,ue,n7,9C,.c,7B,72,ac,98,22,/o,va,2D,n.,_m,B8,A3,8D,n_,12,nE,ca,3A,is,AD,rt,r_,l-,_C,n1,_v,y.,yw,1/,ov,_n,_d,ut,no,ul,sa,CT,_K,SS,_e,F1,ty,ou,nG,tr,s/,il,na,iv,L_,AA,da,Ty,EC,ur,TX,xt,lu,No,r.,SL,Re,sw,_1,om,e/,Pa,xc,_g,_a,X_,/e,vi,ds,ai,==,ts,ni,mg,ic,o/,mt,gm,pk,d.,ch,/p,tu,sp,17,/c,ym,ot,ki,Te,FE,ub,nL,eL,.k,if,he,34,e-,23,ze,rE,iz,St,EE,-p,be,In,ER,67,13,yn,ig,ib,_f,.o,el,55,Un,21,fi,54,mo,mb,gi,_r,Qu,FD,-o,ie,fo,As,7F,48,41,/i,eS,ab,FB,1E,h_,ef,rr,rc,di,b.,ol,im,eg,ap,_l,Se,19,oS,ew,bs,Su,F5,Co,BC,ud,C1,r-,ia,_o,65,.r,sk,o_,ck,CD,Am,9F,un,fa,F6,5F,nk,lo,ev,/f,.t,sE,nO,a_,EN,E4,Di,AC,95,74,1_,1A,us,ly,ll,_b,SA,FC,69,5E,43,um,tT,OS,CE,87,7A,59,44,t-,bl,ad,Or,D5,A_,31,24,t/,ph,mm,f.,ag,RS,Of,It,FA,De,1D,/d,-k,lf,hr,gu,fy,D6,89,6F,4E,/k,w_,cu,br,TE,ST,R_,E8,/O";
    let bigrams = bigrams_bytes.split(|b| *b == b',');
    Arc::new(HashSet::from_iter(bigrams))
});

/// Character class definitions for entropy analysis (zero-copy shared constants)
static STATIC_BASE36_RANGES: LazyLock<Arc<Vec<(u8, u8)>>> = LazyLock::new(|| Arc::new(vec![(b'0', b'9'), (b'A', b'Z')]));
static STATIC_BASE64_RANGES: LazyLock<Arc<Vec<(u8, u8)>>> = LazyLock::new(|| Arc::new(vec![(b'0', b'9'), (b'A', b'Z'), (b'a', b'z')]));

/// Filter matches based on Shannon entropy analysis
/// 
/// This filter validates potential secrets using statistical analysis to distinguish
/// random strings (likely secrets) from non-random text (likely variable names, etc.).
/// 
/// Uses multiple metrics:
/// - Number of distinct values
/// - Character class distribution  
/// - Bigram frequency analysis
/// 
/// Ported from scan-v2 with optimizations for the v3 architecture.
pub struct EntropyFilter {
    /// Minimum entropy threshold for considering a string random
    min_threshold: f64,
}

impl EntropyFilter {
    /// Create a new entropy filter with the specified threshold
    /// 
    /// # Parameters
    /// - `min_threshold`: Minimum probability threshold (e.g., 1e-5 for 1 in 100,000)
    pub fn new(min_threshold: f64) -> Self {
        Self { min_threshold }
    }
    
    /// Determines if a byte sequence is likely a secret based on entropy analysis
    ///
    /// This function uses statistical analysis to determine if a string appears random enough
    /// to be a secret. It combines three metrics:
    /// 1. Number of distinct values
    /// 2. Character class distribution
    /// 3. Bigram frequency analysis
    ///
    /// Returns true if the string appears to be randomly generated (likely a secret)
    pub fn is_likely_secret(&self, data: &[u8]) -> bool {
        let probability = Self::calculate_randomness_probability(data);

        tracing::trace!(
            "Testing '{}' - prob: {:.2e}, threshold: {:.2e}",
            String::from_utf8_lossy(data),
            probability,
            self.min_threshold
        );

        if probability < self.min_threshold {
            tracing::trace!("Failed basic threshold check");
            return false;
        }

        // Additional check: strings without numbers need higher probability
        let contains_number = data.iter().any(|byte| byte.is_ascii_digit());

        if !contains_number && probability < self.min_threshold * 10.0 {
            tracing::trace!(
                "Failed no-numbers threshold check (needs {:.2e})",
                self.min_threshold * 10.0
            );
            return false;
        }

        tracing::trace!("Passed all checks - returning true");
        true
    }

    /// Calculate the probability that a string occurred by random chance
    pub fn calculate_randomness_probability(s: &[u8]) -> f64 {
        let base = if STATIC_HEX_REGEX.is_match(s) {
            16.0
        } else if STATIC_CAP_AND_NUMBERS_REGEX.is_match(s) {
            36.0
        } else {
            64.0
        };

        let mut probability = Self::probability_random_distinct_values(s, base)
                            * Self::probability_random_char_class(s, base);

        if base == 64.0 {
            // Bigrams are only calibrated for base64
            probability *= Self::probability_random_bigrams(s);
        }

        probability
    }

    /// Calculate probability based on bigram analysis using pre-computed bigram set
    fn probability_random_bigrams(s: &[u8]) -> f64 {
        let mut num_bigrams = 0;
        for i in 0..s.len().saturating_sub(1) {
            let bigram = &s[i..=i + 1];
            if STATIC_BIGRAMS.contains(bigram) {
                num_bigrams += 1;
            }
        }

        Self::binomial_probability(
            s.len(),
            num_bigrams,
            (STATIC_BIGRAMS.len() as f64) / (64.0 * 64.0),
        )
    }

    /// Calculate probability based on character class distribution
    fn probability_random_char_class(s: &[u8], base: f64) -> f64 {
        if base == 16.0 {
            Self::probability_random_char_class_aux(s, b'0', b'9', base)
        } else {
            let mut min_probability = f64::INFINITY;

            let char_classes = if base == 36.0 {
                STATIC_BASE36_RANGES.clone() // Cheap Arc clone
            } else {
                STATIC_BASE64_RANGES.clone() // Cheap Arc clone
            };

            for (min, max) in char_classes.iter() {
                let probability = Self::probability_random_char_class_aux(s, *min, *max, base);
                if probability < min_probability {
                    min_probability = probability;
                }
            }
            min_probability
        }
    }

    /// Helper function for character class probability calculation
    fn probability_random_char_class_aux(s: &[u8], min: u8, max: u8, base: f64) -> f64 {
        let count = s.iter().filter(|&&byte| byte >= min && byte <= max).count();
        let num_chars = (max - min + 1) as f64;
        Self::binomial_probability(s.len(), count, num_chars / base)
    }

    /// Calculate binomial probability for statistical analysis
    fn binomial_probability(n: usize, x: usize, p: f64) -> f64 {
        let left_tail = (x as f64) < n as f64 * p;
        let min = if left_tail { 0 } else { x };
        let max = if left_tail { x } else { n };

        let mut total_probability = 0.0;
        for i in min..=max {
            total_probability += Self::factorial(n) / (Self::factorial(n - i) * Self::factorial(i))
                * p.powi(i as i32)
                * (1.0 - p).powi((n - i) as i32);
        }
        total_probability
    }

    /// Calculate factorial for binomial probability calculations
    fn factorial(n: usize) -> f64 {
        let mut result = 1.0;
        for i in 2..=n {
            result *= i as f64;
        }
        result
    }

    /// Calculate probability based on distinct values in the sequence
    fn probability_random_distinct_values(s: &[u8], base: f64) -> f64 {
        let total_possible: f64 = base.powi(s.len() as i32);
        let num_distinct_values = Self::count_distinct_values(s);
        let mut num_more_extreme_outcomes: f64 = 0.0;
        for i in 1..=num_distinct_values {
            num_more_extreme_outcomes += Self::num_possible_outcomes(s.len(), i, base as usize);
        }
        num_more_extreme_outcomes / total_possible
    }

    /// Count distinct byte values in the sequence
    fn count_distinct_values(s: &[u8]) -> usize {
        let mut values_count = HashMap::<u8, usize>::new();
        for &byte in s {
            *values_count.entry(byte).or_insert(0) += 1;
        }
        values_count.len()
    }

    /// Calculate number of possible outcomes for distinct value analysis
    fn num_possible_outcomes(num_values: usize, num_distinct_values: usize, base: usize) -> f64 {
        let mut result = base as f64;
        for i in 1..num_distinct_values {
            result *= (base - i) as f64;
        }
        result * Self::num_distinct_configurations(num_values, num_distinct_values)
    }

    /// Calculate number of distinct configurations
    fn num_distinct_configurations(num_values: usize, num_distinct_values: usize) -> f64 {
        if num_distinct_values == 1 || num_distinct_values == num_values {
            return 1.0;
        }
        Self::num_distinct_configurations_aux(num_distinct_values, 0, num_values - num_distinct_values)
    }

    /// Recursive helper for distinct configurations calculation
    fn num_distinct_configurations_aux(
        num_positions: usize,
        position: usize,
        remaining_values: usize,
    ) -> f64 {
        if remaining_values == 0 {
            return 1.0;
        }
        let mut num_configs = 0.0;
        if position + 1 < num_positions {
            num_configs +=
                Self::num_distinct_configurations_aux(num_positions, position + 1, remaining_values);
        }
        num_configs += (position + 1) as f64
            * Self::num_distinct_configurations_aux(num_positions, position, remaining_values - 1);
        num_configs
    }
}

impl Filter for EntropyFilter {
    type Input = Vec<SecretMatch>;
    type Output = Vec<SecretMatch>;
    
    fn filter(&self, matches: &Vec<SecretMatch>) -> Result<Vec<SecretMatch>> {
        if matches.is_empty() {
            return Ok(Vec::new());
        }

        // Filter matches based on entropy analysis
        let filtered_matches: Vec<SecretMatch> = matches
            .iter()
            .filter(|secret_match| {
                let is_likely = self.is_likely_secret(secret_match.matched_text.as_bytes());
                
                if !is_likely {
                    tracing::debug!(
                        "Filtering out low-entropy match '{}' on line {} (failed entropy check)",
                        secret_match.matched_text,
                        secret_match.location.coordinate.line
                    );
                }
                
                is_likely
            })
            .cloned()
            .collect();

        let filtered_count = matches.len() - filtered_matches.len();
        if filtered_count > 0 {
            tracing::info!(
                "EntropyFilter removed {} low-entropy matches", 
                filtered_count
            );
        }

        Ok(filtered_matches)
    }
    
    fn name(&self) -> &'static str {
        "EntropyFilter"
    }
}

impl Default for EntropyFilter {
    fn default() -> Self {
        // Default threshold from scan-v2: 1e-5 (1 in 100,000)
        Self::new(1e-5)
    }
}

